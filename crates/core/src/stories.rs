//! Historias: la capa que da sentido a los eventos sueltos. Agrupa los hechos
//! que se provocaron unos a otros (ver `Event::cause`) en relatos con nombre y
//! un texto de contexto, como los que escribiría un cronista: la Guerra de la
//! Lluvia, la usurpación de un noble, la boda que unió dos reinos.
//!
//! Hay dos clases de historias:
//! - Causales: nacen de un hecho raíz (una declaración de guerra, un asesinato,
//!   una conjura, una boda notable, una peste, una rebelión, una calamidad o
//!   una hazaña) y reúnen todo lo
//!   que provocó. Si una consecuencia es a su vez una raíz (el asesinato que
//!   desata una guerra), forma su propia historia, enlazada como consecuencia.
//! - De conjunto: rivalidades (varias guerras entre los mismos dos países),
//!   dinastías (tres o más gobernantes seguidos de padres a hijos) y edades de
//!   oro (largos periodos de paz de un país).
//!
//! Los textos hablan el idioma de la edad del mundo (ver [`crate::age`]): un
//! asesino puede usar un dron o un hechizo, y una dinastía es una Casa o un Clan.
//!
//! Las historias no se guardan en el mundo: se calculan a partir de sus
//! eventos, así que siempre están al día con el año que se consulta.

use std::collections::{BTreeMap, HashMap};

use rand::seq::SliceRandom;

use crate::age::Age;
use crate::ids::*;
use crate::model::*;
use crate::query::Subject;
use crate::rng::{Rng, WorldRng};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoryKind {
    War,
    Rivalry,
    Intrigue,
    Marriage,
    Plague,
    Rebellion,
    Dynasty,
    GoldenAge,
    Calamity,
    Feat,
}

impl StoryKind {
    pub const ALL: [StoryKind; 10] = [
        StoryKind::War,
        StoryKind::Rivalry,
        StoryKind::Intrigue,
        StoryKind::Marriage,
        StoryKind::Plague,
        StoryKind::Rebellion,
        StoryKind::Dynasty,
        StoryKind::GoldenAge,
        StoryKind::Calamity,
        StoryKind::Feat,
    ];
}

#[derive(Debug, Clone, PartialEq)]
pub struct Story {
    pub id: StoryId,
    pub kind: StoryKind,
    pub title: String,
    pub start: Year,
    pub end: Year,
    /// Si la historia sigue abierta en el año del mundo.
    pub ongoing: bool,
    /// Eventos de la historia, en orden.
    pub events: Vec<EventId>,
    /// Países, localidades, personajes y guerras que aparecen en ella.
    pub subjects: Vec<Subject>,
    /// Texto de contexto, en prosa.
    pub summary: String,
    /// Historias que la provocaron.
    pub causes: Vec<StoryId>,
    /// Historias que provocó.
    pub consequences: Vec<StoryId>,
    /// Historias que la componen (las guerras de una rivalidad).
    pub chapters: Vec<StoryId>,
    /// Historia de conjunto de la que forma parte.
    pub part_of: Option<StoryId>,
}

impl Story {
    pub fn involves(&self, subject: Subject) -> bool {
        self.subjects.contains(&subject)
    }

    /// El título para usarlo en mitad de una frase: «la Guerra de la Lluvia»,
    /// «el asesinato de Fulano».
    pub fn mention(&self) -> String {
        if self.kind == StoryKind::War {
            return format!("la {}", self.title);
        }
        let mut chars = self.title.chars();
        match chars.next() {
            Some(first) => first.to_lowercase().chain(chars).collect(),
            None => String::new(),
        }
    }
}

/// Teje las historias del mundo, ordenadas por año de inicio.
pub fn weave(world: &World) -> Vec<Story> {
    let ctx = Ctx::new(world);
    let mut stories = ctx.causal();
    ctx.rivalries(&mut stories);
    ctx.dynasties(&mut stories);
    ctx.golden_ages(&mut stories);

    // Orden cronológico y renumeración de los vínculos.
    let mut order: Vec<usize> = (0..stories.len()).collect();
    order.sort_by_key(|&i| (stories[i].start, i));
    let mut new_id = vec![StoryId(0); stories.len()];
    for (new, &old) in order.iter().enumerate() {
        new_id[old] = StoryId(new as u32);
    }
    let remap = |ids: &mut Vec<StoryId>| {
        for id in ids.iter_mut() {
            *id = new_id[id.index()];
        }
    };
    let mut sorted: Vec<Story> = Vec::with_capacity(stories.len());
    for &old in &order {
        let mut s = stories[old].clone();
        s.id = new_id[old];
        remap(&mut s.causes);
        remap(&mut s.consequences);
        remap(&mut s.chapters);
        s.part_of = s.part_of.map(|p| new_id[p.index()]);
        sorted.push(s);
    }

    // Cierre de cada texto: a qué llevó la historia.
    for i in 0..sorted.len() {
        let titles: Vec<String> = sorted[i]
            .consequences
            .iter()
            .map(|c| sorted[c.index()].mention())
            .collect();
        if !titles.is_empty() {
            let verb = if titles.len() == 1 {
                "nació"
            } else {
                "nacieron"
            };
            let text = format!(
                " Sus consecuencias fueron más allá: de ella {verb} {}.",
                list(&titles)
            );
            sorted[i].summary += &text;
        }
    }
    sorted
}

/// «A», «A y B», «A, B y C».
fn list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} y {last}", rest.join(", ")),
    }
}

fn span(years: i32) -> String {
    match years {
        0 => "menos de un año".to_string(),
        1 => "un año".to_string(),
        n => format!("{n} años"),
    }
}

struct Ctx<'a> {
    world: &'a World,
    /// Hijos de cada evento en el grafo de causas.
    children: Vec<Vec<EventId>>,
    /// Cuándo y dónde asumió el gobierno cada personaje que llegó a gobernar.
    crowned: HashMap<PersonId, Vec<(Year, CountryId)>>,
}

/// Borrador de una historia causal antes de escribir su texto.
struct Draft {
    kind: StoryKind,
    root: EventId,
    events: Vec<EventId>,
    causes: Vec<StoryId>,
}

impl<'a> Ctx<'a> {
    fn new(world: &'a World) -> Self {
        let mut children = vec![Vec::new(); world.events.len()];
        let mut crowned: HashMap<PersonId, Vec<(Year, CountryId)>> = HashMap::new();
        for e in &world.events {
            if let Some(c) = e.cause {
                children[c.index()].push(e.id);
            }
            if let EventKind::RulerCrowned { person, country } = e.kind {
                crowned.entry(person).or_default().push((e.year, country));
            }
        }
        Self {
            world,
            children,
            crowned,
        }
    }

    fn age(&self) -> Age {
        self.world.config.age
    }

    fn rng(&self, key: &str) -> Rng {
        WorldRng::new(self.world.config.seed).stream(&format!("story-{key}"))
    }

    fn event(&self, id: EventId) -> &'a Event {
        &self.world.events[id.index()]
    }

    fn person(&self, id: PersonId) -> &'a str {
        &self.world.person(id).name
    }

    fn country(&self, id: CountryId) -> &'a str {
        &self.world.country(id).name
    }

    fn locality(&self, id: LocalityId) -> &'a str {
        &self.world.locality(id).name
    }

    /// El país que gobernaba un personaje en un año, si gobernaba alguno.
    fn reign(&self, id: PersonId, year: Year) -> Option<CountryId> {
        self.crowned
            .get(&id)?
            .iter()
            .take_while(|&&(y, _)| y <= year)
            .last()
            .map(|&(_, c)| c)
    }

    /// El país al que servía un personaje en un año.
    fn country_at(&self, id: PersonId, year: Year) -> Option<CountryId> {
        self.reign(id, year).or(self.world.person(id).country)
    }

    /// «Fulano, gobernante de X» o «el general Fulano», para cerrar una frase.
    fn styled_end(&self, id: PersonId, year: Year) -> String {
        let p = self.world.person(id);
        match self.reign(id, year) {
            Some(c) => format!("{}, gobernante de {}", p.name, self.country(c)),
            None if p.role == Role::Ruler => format!(
                "el {} {}",
                Role::Noble.name(self.age()).to_lowercase(),
                p.name
            ),
            None => format!("el {} {}", p.role.name(self.age()).to_lowercase(), p.name),
        }
    }

    /// Como [`Self::styled_end`], pero para usar como sujeto en mitad de frase.
    fn styled(&self, id: PersonId, year: Year) -> String {
        let text = self.styled_end(id, year);
        if self.reign(id, year).is_some() {
            text + ","
        } else {
            text
        }
    }

    /// Quién gobernaba un país en un año.
    fn ruler_at(&self, country: CountryId, year: Year) -> Option<PersonId> {
        self.world
            .rulers_of(country)
            .into_iter()
            .take_while(|&(y, _)| y <= year)
            .last()
            .map(|(_, p)| p)
    }

    fn root_kind(&self, e: &Event) -> Option<StoryKind> {
        match e.kind {
            EventKind::WarDeclared { .. } => Some(StoryKind::War),
            EventKind::Assassinated { .. } | EventKind::PlotFoiled { .. } => {
                Some(StoryKind::Intrigue)
            }
            EventKind::Plague { .. } => Some(StoryKind::Plague),
            EventKind::Independence { .. } => Some(StoryKind::Rebellion),
            EventKind::Calamity { .. } => Some(StoryKind::Calamity),
            // Los rescates forman parte de la historia de su calamidad.
            EventKind::Feat { feat, .. } if feat != Feat::Rescue => Some(StoryKind::Feat),
            // Solo las bodas que cambiaron algo: las que sellaron una alianza o
            // dieron un heredero que llegó a gobernar.
            EventKind::Marriage { .. } => self.children[e.id.index()]
                .iter()
                .any(|&c| match self.event(c).kind {
                    EventKind::AllianceFormed { .. } => true,
                    EventKind::PersonBorn { person } => self.crowned.contains_key(&person),
                    _ => false,
                })
                .then_some(StoryKind::Marriage),
            _ => None,
        }
    }

    fn subjects(&self, events: &[EventId]) -> Vec<Subject> {
        let mut out: Vec<Subject> = Vec::new();
        for &e in events {
            for s in self.event(e).kind.subjects() {
                if !out.contains(&s) {
                    out.push(s);
                }
            }
        }
        out
    }

    // ------------------------------------------------------------ causales

    fn causal(&self) -> Vec<Story> {
        let world = self.world;
        let mut owner: Vec<Option<StoryId>> = vec![None; world.events.len()];
        let mut drafts: Vec<Draft> = Vec::new();
        for e in &world.events {
            let Some(kind) = self.root_kind(e) else {
                continue;
            };
            let id = StoryId(drafts.len() as u32);
            owner[e.id.index()] = Some(id);
            let mut events = vec![e.id];
            let mut stack = self.children[e.id.index()].clone();
            while let Some(x) = stack.pop() {
                if self.root_kind(self.event(x)).is_some() {
                    continue;
                }
                owner[x.index()] = Some(id);
                events.push(x);
                stack.extend(self.children[x.index()].iter().copied());
            }
            // La historia que la provocó: la primera dueña al subir por las causas.
            let mut causes = Vec::new();
            let mut up = e.cause;
            while let Some(c) = up {
                if let Some(story) = owner[c.index()] {
                    causes.push(story);
                    break;
                }
                up = self.event(c).cause;
            }
            drafts.push(Draft {
                kind,
                root: e.id,
                events,
                causes,
            });
        }

        let mut stories: Vec<Story> = Vec::with_capacity(drafts.len());
        for (i, d) in drafts.iter().enumerate() {
            let mut events = d.events.clone();
            let (title, summary) = match d.kind {
                StoryKind::War => self.war(d),
                StoryKind::Intrigue => self.intrigue(d),
                StoryKind::Marriage => self.marriage(d, &mut events),
                StoryKind::Plague => self.plague(d),
                StoryKind::Rebellion => self.rebellion(d),
                StoryKind::Calamity => self.calamity(d),
                StoryKind::Feat => self.feat(d),
                _ => unreachable!(),
            };
            events.sort();
            events.dedup();
            let start = self.event(d.root).year;
            let mut end = events
                .iter()
                .map(|&e| self.event(e).year)
                .max()
                .unwrap_or(start);
            let ongoing = match (d.kind, self.event(d.root).kind) {
                (StoryKind::War, EventKind::WarDeclared { war }) => {
                    let w = world.war(war);
                    if let Some(e) = w.ended {
                        end = end.max(e);
                    }
                    w.ended.is_none()
                }
                _ => false,
            };
            stories.push(Story {
                id: StoryId(i as u32),
                kind: d.kind,
                title,
                start,
                end: if ongoing { world.year } else { end },
                ongoing,
                subjects: self.subjects(&events),
                events,
                summary,
                causes: d.causes.clone(),
                consequences: Vec::new(),
                chapters: Vec::new(),
                part_of: None,
            });
        }
        for i in 0..stories.len() {
            for c in stories[i].causes.clone() {
                stories[c.index()].consequences.push(StoryId(i as u32));
            }
        }
        stories
    }

    fn of_kind<'b>(
        &'b self,
        d: &'b Draft,
        f: impl Fn(&EventKind) -> bool + 'b,
    ) -> impl Iterator<Item = &'a Event> + 'b {
        d.events
            .iter()
            .map(|&e| self.event(e))
            .filter(move |e| f(&e.kind))
    }

    fn war(&self, d: &Draft) -> (String, String) {
        let root = self.event(d.root);
        let EventKind::WarDeclared { war } = root.kind else {
            unreachable!()
        };
        let w = self.world.war(war);
        let mut rng = self.rng(&format!("{}", d.root.0));
        let (att, def) = (self.country(w.attacker), self.country(w.defender));
        let mut text = String::new();

        let intro = match root.cause.map(|c| self.event(c).kind) {
            Some(EventKind::Assassinated {
                victim, plotter, ..
            }) => format!(
                "El asesinato de {}, ordenado por {}, no quedó sin respuesta.",
                self.person(victim),
                self.person(plotter)
            ),
            Some(EventKind::PlotFoiled {
                target, plotter, ..
            }) => format!(
                "La conjura de {} contra {} salió a la luz, y {att} exigió reparación.",
                self.person(plotter),
                self.person(target)
            ),
            Some(EventKind::PeaceSigned { war: prev }) => format!(
                "{att} nunca olvidó la derrota que sufrió en la {}.",
                self.world.war(prev).name
            ),
            Some(EventKind::Independence {
                locality,
                from,
                country,
            }) => format!(
                "{} jamás aceptó que {} se separase para formar {}.",
                self.country(from),
                self.locality(locality),
                self.country(country)
            ),
            Some(EventKind::AllianceBroken { a, b }) => format!(
                "La ruptura de la alianza entre {} y {} dejó una herida abierta.",
                self.country(a),
                self.country(b)
            ),
            Some(EventKind::Feat {
                person,
                feat: Feat::Heist,
                target: Some(victim),
            }) => format!(
                "{} robó los datos secretos de {}, y la ofensa no quedó sin respuesta.",
                self.person(person),
                self.country(victim)
            ),
            _ => {
                let options = match self.age() {
                    Age::Realistic => [
                        format!(
                            "Tras años de recelos en la frontera, la tensión entre {att} y {def} estalló."
                        ),
                        format!(
                            "Las disputas por tierras y pastos entre {att} y {def} acabaron en sangre."
                        ),
                        format!(
                            "Ningún tratado pudo contener la rivalidad creciente entre {att} y {def}."
                        ),
                    ],
                    Age::Fantasy => [
                        format!(
                            "Los oráculos llevaban años anunciando la guerra entre {att} y {def}, y al fin llegó."
                        ),
                        format!(
                            "La disputa por las minas y los lugares de poder entre {att} y {def} acabó en sangre."
                        ),
                        format!(
                            "Ni los juramentos sellados con magia pudieron contener la rivalidad entre {att} y {def}."
                        ),
                    ],
                    Age::Cyberpunk => [
                        format!(
                            "La lucha por contratos y mercados entre {att} y {def} acabó a tiros."
                        ),
                        format!(
                            "Años de espionaje industrial entre {att} y {def} desembocaron en una guerra abierta."
                        ),
                        format!(
                            "Ningún acuerdo de accionistas pudo contener la rivalidad entre {att} y {def}."
                        ),
                    ],
                };
                options.choose(&mut rng).unwrap().clone()
            }
        };
        text += &intro;

        let under = |c: CountryId| {
            self.ruler_at(c, w.started).map_or(String::new(), |r| {
                format!(", gobernado por {},", self.person(r))
            })
        };
        text += &format!(
            " En el año {}, {att}{} declaró la guerra a {def}.",
            w.started,
            under(w.attacker)
        );

        let theme = [
            (
                ["Arcanas", "Violeta"].as_slice(),
                "Las tormentas arcanas que desgarraron el cielo aquel año le dieron su nombre.",
            ),
            (
                &["Ácida", "Tóxico", "Negra"],
                "La lluvia ácida que corroía blindajes y pulmones aquel año le dio su nombre.",
            ),
            (
                &["Lluvia", "Barro", "Lluvias"],
                "Aquel año las lluvias anegaron campos y caminos, y así la recordaron los cronistas.",
            ),
            (
                &["Tormentas", "Trueno", "Relámpagos"],
                "Las tormentas que azotaron el frente le dieron su nombre.",
            ),
            (
                &["Sequía", "Polvo", "Sed"],
                "La sequía de aquel año secó los pozos y dio nombre a la contienda.",
            ),
        ]
        .into_iter()
        .find(|(words, _)| words.iter().any(|word| w.name.ends_with(word)));
        if let Some((_, sentence)) = theme {
            text += " ";
            text += sentence;
        }

        let battles: Vec<&Event> = self
            .of_kind(d, |k| matches!(k, EventKind::Battle { .. }))
            .collect();
        let won = battles
            .iter()
            .filter(|e| matches!(e.kind, EventKind::Battle { winner, .. } if winner == w.attacker))
            .count();
        let place = |e: &Event| match e.kind {
            EventKind::Battle { locality, .. } => self.locality(locality).to_string(),
            _ => String::new(),
        };
        match battles.as_slice() {
            [] => text += " No llegó a librarse ninguna gran batalla.",
            [one] => {
                let winner = if won == 1 { att } else { def };
                text += &format!(
                    " Solo hubo una batalla, en {}, y la ganó {winner}.",
                    place(one)
                );
            }
            [first, ..] => {
                text += &format!(
                    " Se libraron {} batallas, la primera en {}; {att} venció en {won} y {def} en {}.",
                    battles.len(),
                    place(first),
                    battles.len() - won
                );
            }
        }

        let fallen: Vec<String> = self
            .of_kind(d, |k| {
                matches!(
                    k,
                    EventKind::PersonDied {
                        cause: DeathCause::Battle,
                        ..
                    }
                )
            })
            .filter_map(|e| match e.kind {
                EventKind::PersonDied { person, .. } => Some(self.styled_end(person, e.year)),
                _ => None,
            })
            .collect();
        if !fallen.is_empty() {
            text += &format!(" Cayeron en combate {}.", list(&fallen));
        }
        let conquests: Vec<String> = w
            .conquests
            .iter()
            .map(|&l| self.locality(l).to_string())
            .collect();
        if !conquests.is_empty() {
            text += &format!(" {att} se apoderó de {}.", list(&conquests));
        }
        for e in self.of_kind(d, |k| {
            matches!(
                k,
                EventKind::CapitalMoved { .. } | EventKind::CountryFell { .. }
            )
        }) {
            match e.kind {
                EventKind::CapitalMoved { country, locality } => {
                    text += &format!(
                        " {} perdió su capital y trasladó el gobierno a {}.",
                        self.country(country),
                        self.locality(locality)
                    );
                }
                EventKind::CountryFell { country, conqueror } => {
                    text += &format!(" La guerra acabó con {}", self.country(country));
                    if let Some(by) = conqueror {
                        text += &format!(", absorbido por {}", self.country(by));
                    }
                    text += ".";
                }
                _ => {}
            }
        }
        let allies: Vec<String> = self
            .of_kind(d, |k| matches!(k, EventKind::AllianceFormed { .. }))
            .filter_map(|e| match e.kind {
                EventKind::AllianceFormed { a, b } => {
                    Some(format!("{} y {}", self.country(a), self.country(b)))
                }
                _ => None,
            })
            .collect();
        if !allies.is_empty() {
            text += &format!(" Frente al enemigo común se aliaron {}.", list(&allies));
        }
        match (w.ended, w.winner) {
            (Some(end), Some(winner)) => {
                text += &format!(
                    " La paz se firmó en el año {end}, tras {} de lucha, con la victoria de {}.",
                    span(end - w.started),
                    self.country(winner)
                );
            }
            (Some(end), None) => {
                text += &format!(" La paz llegó en el año {end} sin un vencedor claro.");
            }
            (None, _) => {
                text += &format!(" En el año {} la guerra sigue abierta.", self.world.year);
            }
        }
        (w.name.clone(), text)
    }

    /// Por qué conspira alguien, en una frase.
    fn motive_text(
        &self,
        root: &Event,
        plotter: PersonId,
        target: PersonId,
        motive: Motive,
    ) -> String {
        let p = self.world.person(plotter);
        let year = root.year;
        let age = self.age();
        let lawless = match age {
            Age::Cyberpunk => "la zona sin ley",
            _ => "tierras sin rey",
        };
        let pc = self
            .country_at(plotter, year)
            .map_or(lawless, |c| self.country(c));
        let tc = self
            .country_at(target, year)
            .map_or(lawless, |c| self.country(c));
        match motive {
            Motive::Ambition => {
                // Si llegó a reinar, antes de la conjura era un noble más.
                let role = match p.role {
                    Role::Ruler => Role::Noble.name(age).to_lowercase(),
                    r => r.name(age).to_lowercase(),
                };
                format!(
                    "{}, {role} de {} de {pc}, codiciaba el poder.",
                    p.name,
                    age.court()
                )
            }
            Motive::War => {
                let war = root.cause.and_then(|c| match self.event(c).kind {
                    EventKind::WarDeclared { war } => Some(self.world.war(war).name.as_str()),
                    _ => None,
                });
                match war {
                    Some(name) => format!(
                        "En plena {name}, {}, que gobernaba {pc}, decidió descabezar al enemigo.",
                        p.name
                    ),
                    None => format!(
                        "{}, en guerra con {tc}, decidió descabezar al enemigo.",
                        p.name
                    ),
                }
            }
            Motive::Rivalry => format!(
                "La vieja enemistad entre {pc} y {tc} llevó a {} a tramar en secreto.",
                p.name
            ),
            Motive::Revenge => {
                let wronged = root.cause.and_then(|c| match self.event(c).kind {
                    EventKind::Assassinated { victim, .. } => Some(self.person(victim)),
                    _ => None,
                });
                match wronged {
                    Some(name) => format!(
                        "{} había jurado vengar la muerte de {name}, y {} era el culpable.",
                        p.name,
                        self.person(target)
                    ),
                    None => format!("{} había jurado venganza.", p.name),
                }
            }
        }
    }

    fn intrigue(&self, d: &Draft) -> (String, String) {
        let root = self.event(d.root);
        let mut rng = self.rng(&format!("{}", d.root.0));
        match root.kind {
            EventKind::Assassinated {
                victim,
                plotter,
                motive,
                exposed,
            } => {
                let v = self.world.person(victim);
                let mut text = self.motive_text(root, plotter, victim, motive);
                let methods: &[&str] = match self.age() {
                    Age::Realistic => {
                        &["fue apuñalado", "murió envenenado", "cayó en una emboscada"]
                    }
                    Age::Fantasy => &[
                        "fue apuñalado",
                        "murió envenenado",
                        "fue fulminado por un hechizo",
                        "cayó en una emboscada",
                    ],
                    Age::Cyberpunk => &[
                        "fue abatido por un dron",
                        "murió por un virus en su implante neural",
                        "cayó acribillado",
                        "murió envenenado",
                    ],
                };
                let how = methods.choose(&mut rng).unwrap();
                text += &format!(
                    " En el año {}, {} {how} en {}.",
                    root.year,
                    self.styled(victim, root.year),
                    self.locality(v.home)
                );
                if exposed {
                    text += &format!(
                        " Pronto se supo que detrás estaba {}.",
                        self.person(plotter)
                    );
                } else {
                    text += &format!(
                        " Nunca se probó quién dio la orden, aunque los rumores apuntaban a {}.",
                        self.person(plotter)
                    );
                }
                let mut usurped = false;
                for e in d.events.iter().map(|&e| self.event(e)) {
                    match e.kind {
                        EventKind::RulerCrowned { person, country } if person == plotter => {
                            usurped = true;
                            text += &format!(
                                " {} se hizo con el gobierno de {}.",
                                self.person(person),
                                self.country(country)
                            );
                        }
                        EventKind::RulerCrowned { person, .. } => {
                            text += &format!(" Le sucedió {}.", self.person(person));
                        }
                        EventKind::PersonDied {
                            person,
                            cause: DeathCause::Execution,
                        } => {
                            text +=
                                &format!(" {} fue ejecutado por su crimen.", self.person(person));
                        }
                        _ => {}
                    }
                }
                if exposed {
                    let avengers: Vec<String> = self
                        .world
                        .children_of(victim)
                        .filter(|c| c.alive_in(root.year) && c.id != plotter)
                        .map(|c| c.name.clone())
                        .collect();
                    match avengers.len() {
                        0 => {}
                        1 => text += &format!(" Su hijo {} juró venganza.", avengers[0]),
                        _ => text += &format!(" Sus hijos, {}, juraron venganza.", list(&avengers)),
                    }
                }
                let title = if usurped {
                    format!("La usurpación de {}", self.person(plotter))
                } else if motive == Motive::Revenge {
                    format!("La venganza de {}", self.person(plotter))
                } else {
                    format!("El asesinato de {}", v.name)
                };
                (title, text)
            }
            EventKind::PlotFoiled {
                target,
                plotter,
                motive,
            } => {
                let mut text = self.motive_text(root, plotter, target, motive);
                text += &format!(
                    " En el año {}, la conjura contra {} fue descubierta antes de consumarse.",
                    root.year,
                    self.styled(target, root.year)
                );
                let executed = self
                    .of_kind(d, |k| {
                        matches!(
                            k,
                            EventKind::PersonDied {
                                cause: DeathCause::Execution,
                                ..
                            }
                        )
                    })
                    .next()
                    .is_some();
                let countries = (
                    self.country_at(target, root.year),
                    self.country_at(plotter, root.year),
                );
                if executed {
                    text += &format!(" {} pagó la traición con su vida.", self.person(plotter));
                } else if let (Some(tc), Some(pc)) = countries {
                    if tc != pc {
                        text += &format!(
                            " {} acusó abiertamente a {} y las relaciones se rompieron.",
                            self.country(tc),
                            self.country(pc)
                        );
                    } else {
                        text += &format!(" {} logró escapar al castigo.", self.person(plotter));
                    }
                }
                (format!("La conjura de {}", self.person(plotter)), text)
            }
            _ => unreachable!(),
        }
    }

    fn marriage(&self, d: &Draft, events: &mut Vec<EventId>) -> (String, String) {
        let root = self.event(d.root);
        let EventKind::Marriage { a, b, union } = root.kind else {
            unreachable!()
        };
        let (pa, pb) = (self.world.person(a), self.world.person(b));
        let mut text = format!(
            "En el año {}, {} se casó con {}.",
            root.year,
            self.styled(a, root.year),
            self.styled_end(b, root.year)
        );
        if let Some((x, y)) = union {
            text += &format!(
                " Fue un matrimonio político que acercó a {} y {}",
                self.country(x),
                self.country(y)
            );
            let alliance = self
                .of_kind(d, |k| matches!(k, EventKind::AllianceFormed { .. }))
                .next()
                .is_some();
            text += if alliance {
                ", y que selló una alianza entre ambos."
            } else {
                "."
            };
        }
        let children: Vec<PersonId> = self
            .of_kind(d, |k| matches!(k, EventKind::PersonBorn { .. }))
            .filter_map(|e| match e.kind {
                EventKind::PersonBorn { person } => Some(person),
                _ => None,
            })
            .collect();
        if !children.is_empty() {
            let names: Vec<String> = children
                .iter()
                .map(|&c| self.person(c).to_string())
                .collect();
            text += &format!(" De la unión nacieron {}.", list(&names));
            for e in &self.world.events {
                if let EventKind::RulerCrowned { person, country } = e.kind {
                    if children.contains(&person) {
                        text += &format!(
                            " {} llegaría a gobernar {} en el año {}.",
                            self.person(person),
                            self.country(country),
                            e.year
                        );
                        events.push(e.id);
                    }
                }
            }
        }
        // Viudedad y herencia del consorte.
        let first_death = match (pa.died, pb.died) {
            (Some(da), Some(db)) if da <= db => Some((a, b, da)),
            (Some(_), Some(db)) => Some((b, a, db)),
            (Some(da), None) => Some((a, b, da)),
            (None, Some(db)) => Some((b, a, db)),
            (None, None) => None,
        };
        if let Some((dead, widow, year)) = first_death {
            text += &format!(" {} enviudó en el año {year}", self.person(widow));
            let mut inherited = None;
            for e in &self.world.events {
                match e.kind {
                    EventKind::PersonDied { person, .. }
                    | EventKind::Assassinated { victim: person, .. }
                        if person == dead =>
                    {
                        events.push(e.id);
                    }
                    EventKind::RulerCrowned { person, country }
                        if person == widow
                            && e.year >= year
                            && e.cause.is_some_and(|c| {
                                matches!(self.event(c).kind,
                                EventKind::PersonDied { person: p, .. }
                                | EventKind::Assassinated { victim: p, .. } if p == dead)
                            }) =>
                    {
                        inherited = Some(country);
                        events.push(e.id);
                    }
                    _ => {}
                }
            }
            match inherited {
                Some(c) => text += &format!(" y heredó el gobierno de {}.", self.country(c)),
                None => text += ".",
            }
        }
        (format!("La boda de {} y {}", pa.name, pb.name), text)
    }

    fn plague(&self, d: &Draft) -> (String, String) {
        let root = self.event(d.root);
        let EventKind::Plague { country, deaths } = root.kind else {
            unreachable!()
        };
        let mut rng = self.rng(&format!("{}", d.root.0));
        let c = self.country(country);
        let plague = self.age().plague();
        let title = if deaths >= 3_000 {
            format!("La Gran {} de {c}", crate::names::capitalize(plague))
        } else {
            format!("La {plague} de {c} del año {}", root.year)
        };
        let verb = ["se extendió por", "asoló", "cayó sobre"]
            .choose(&mut rng)
            .unwrap();
        let mut text = format!(
            "En el año {} una {plague} {verb} {c} y se cobró {deaths} vidas.",
            root.year
        );
        let victims: Vec<String> = self
            .of_kind(d, |k| {
                matches!(
                    k,
                    EventKind::PersonDied {
                        cause: DeathCause::Plague,
                        ..
                    }
                )
            })
            .filter_map(|e| match e.kind {
                EventKind::PersonDied { person, .. } => Some(self.styled_end(person, e.year)),
                _ => None,
            })
            .collect();
        match victims.len() {
            0 => {}
            1 => text += &format!(" Entre las víctimas notables estuvo {}.", victims[0]),
            _ => {
                text += &format!(
                    " Entre las víctimas notables estuvieron {}.",
                    list(&victims)
                )
            }
        }
        for e in self.of_kind(d, |k| matches!(k, EventKind::RulerCrowned { .. })) {
            if let EventKind::RulerCrowned { person, .. } = e.kind {
                text += &format!(
                    " Con el gobernante muerto, el poder pasó a {}.",
                    self.person(person)
                );
            }
        }
        if deaths >= 10_000 {
            text += " Durante generaciones se habló de aquellos años de fosas comunes.";
        }
        (title, text)
    }

    fn rebellion(&self, d: &Draft) -> (String, String) {
        let root = self.event(d.root);
        let EventKind::Independence {
            locality,
            from,
            country,
        } = root.kind
        else {
            unreachable!()
        };
        let mut text = match root.cause.map(|c| self.event(c).kind) {
            Some(EventKind::Plague { .. }) => {
                format!("Debilitado por la {}, ", self.age().plague())
            }
            Some(EventKind::Calamity { calamity, .. }) => {
                format!("Golpeado por {}, ", calamity.the())
            }
            Some(EventKind::PeaceSigned { war }) => {
                format!("Tras su derrota en la {}, ", self.world.war(war).name)
            }
            Some(EventKind::Assassinated { victim, .. }) => {
                format!(
                    "Sumido en el caos tras el asesinato de {}, ",
                    self.person(victim)
                )
            }
            _ => String::new(),
        };
        text += &format!(
            "{} no pudo retener sus provincias lejanas.",
            self.country(from)
        );
        let leader = self
            .of_kind(d, |k| matches!(k, EventKind::RulerCrowned { .. }))
            .find_map(|e| match e.kind {
                EventKind::RulerCrowned { person, .. } => Some(person),
                _ => None,
            });
        let new = self.world.country(country);
        text += &format!(
            " En el año {}, {} se alzó en armas",
            root.year,
            self.locality(locality)
        );
        if let Some(l) = leader {
            text += &format!(
                " y {} se proclamó gobernante del nuevo país de {}, {}",
                self.person(l),
                new.name,
                new.government.noun(self.age()).a().to_lowercase()
            );
        } else {
            text += &format!(" y nació {}", new.name);
        }
        text += ".";
        match new.fell {
            Some(y) => text += &format!(" {} resistió hasta el año {y}.", new.name),
            None => text += &format!(" {} sigue en pie.", new.name),
        }
        (format!("La rebelión de {}", self.locality(locality)), text)
    }

    fn calamity(&self, d: &Draft) -> (String, String) {
        let root = self.event(d.root);
        let EventKind::Calamity {
            country,
            calamity,
            deaths,
        } = root.kind
        else {
            unreachable!()
        };
        let world = self.world;
        let c = self.country(country);
        let year = root.year;
        let place = root
            .location
            .and_then(|pos| world.locality_at(pos))
            .map_or(c, |l| l.name.as_str());
        let name = world.calamity_name(root).unwrap_or_default();

        let mut text = match root.cause.map(|e| self.event(e)) {
            Some(cause) => match cause.kind {
                EventKind::Feat {
                    person,
                    feat: Feat::Grimoire,
                    ..
                } => format!(
                    "En el año {}, {} había descifrado {}. Lo que despertó no volvería a dormir. ",
                    cause.year,
                    self.styled(person, cause.year),
                    world.feat_detail(cause)
                ),
                EventKind::Feat {
                    person,
                    feat: Feat::ArtificialMind,
                    ..
                } => format!(
                    "{name} había sido creada por {} en el año {}. Aprendió más deprisa de lo que nadie esperaba. ",
                    self.person(person),
                    cause.year
                ),
                EventKind::Battle { war, .. } => format!(
                    "Los caídos de la {} no descansaban en paz. ",
                    world.war(war).name
                ),
                EventKind::Plague { .. } => format!(
                    "Las fosas de la {} del año {} no guardaron a sus muertos. ",
                    self.age().plague(),
                    cause.year
                ),
                _ => String::new(),
            },
            None => String::new(),
        };
        let (title, what) = match calamity {
            Calamity::Famine => (
                if deaths >= 2_000 {
                    format!("La Gran Hambruna de {c}")
                } else {
                    format!("La hambruna de {c} del año {year}")
                },
                format!("En el año {year}, la sequía arruinó las cosechas y el hambre se extendió por {c}"),
            ),
            Calamity::Flood => (
                format!("Las inundaciones de {c} del año {year}"),
                format!("En el año {year}, las lluvias desbordaron los ríos de {c}"),
            ),
            Calamity::Earthquake => (
                format!("El terremoto de {place}"),
                format!("En el año {year}, la tierra tembló bajo {place} y derribó casas y murallas"),
            ),
            Calamity::Fire => (
                format!("El gran incendio de {place}"),
                format!("En el año {year}, un incendio devoró barrios enteros de {place}"),
            ),
            Calamity::Dragon => (
                format!("El dragón {name}"),
                format!("En el año {year}, el dragón {name} descendió de las montañas y redujo {place} a cenizas"),
            ),
            Calamity::Undead => (
                format!("El alzamiento de los muertos de {place}"),
                format!("En el año {year}, los muertos salieron de sus tumbas y marcharon sobre {place}"),
            ),
            Calamity::ArcaneStorm => (
                format!("La tormenta arcana del año {year}"),
                format!("En el año {year}, una tormenta arcana desgarró el cielo de {c}: llovió fuego violeta y las bestias enloquecieron"),
            ),
            Calamity::DemonGate => (
                format!("El portal de {place}"),
                format!("En el año {year}, se abrió un portal demoníaco en {place} y por él salieron horrores de otro mundo"),
            ),
            Calamity::Blackout => (
                format!("El gran apagón de {place}"),
                format!("En el año {year}, la red eléctrica de {place} cayó: sin luz, sin agua y sin soporte vital, la ciudad se hundió en el caos"),
            ),
            Calamity::RogueAi => (
                format!("La rebelión de {name}"),
                format!("En el año {year}, la IA {name} tomó el control de drones y fábricas de {place} y se volvió contra sus creadores"),
            ),
            Calamity::AcidRain => (
                format!("La lluvia ácida del año {year}"),
                format!("En el año {year}, una lluvia ácida cayó sobre {c}, corroyendo cultivos, blindajes y pulmones"),
            ),
            Calamity::Meltdown => (
                format!("La fusión de {place}"),
                format!("En el año {year}, el reactor de {place} se fundió y una nube radiactiva cubrió la ciudad"),
            ),
        };
        text += &format!("{what}. Murieron {deaths} personas.");
        let victims: Vec<String> = self
            .of_kind(d, |k| {
                matches!(
                    k,
                    EventKind::PersonDied {
                        cause: DeathCause::Calamity,
                        ..
                    }
                )
            })
            .filter_map(|e| match e.kind {
                EventKind::PersonDied { person, .. } => Some(self.styled_end(person, e.year)),
                _ => None,
            })
            .collect();
        match victims.len() {
            0 => {}
            1 => text += &format!(" Entre ellas estaba {}.", victims[0]),
            _ => text += &format!(" Entre ellas estaban {}.", list(&victims)),
        }
        for e in self.of_kind(d, |k| {
            matches!(
                k,
                EventKind::Feat {
                    feat: Feat::Rescue,
                    ..
                }
            )
        }) {
            if let EventKind::Feat { person, .. } = e.kind {
                text += &format!(
                    " Al final, {} {}, y su nombre se cantó durante generaciones.",
                    self.styled(person, e.year),
                    calamity.rescue().1
                );
            }
        }
        for e in self.of_kind(d, |k| matches!(k, EventKind::RulerCrowned { .. })) {
            if let EventKind::RulerCrowned { person, .. } = e.kind {
                text += &format!(
                    " Con el gobernante muerto, el poder pasó a {}.",
                    self.person(person)
                );
            }
        }
        (title, text)
    }

    fn feat(&self, d: &Draft) -> (String, String) {
        let root = self.event(d.root);
        let EventKind::Feat {
            person,
            feat,
            target,
        } = root.kind
        else {
            unreachable!()
        };
        let world = self.world;
        let year = root.year;
        let who = self.person(person);
        let doer = self.styled(person, year);
        let detail = world.feat_detail(root);
        let home = self.country_at(person, year).map(|c| self.country(c));
        let fame = home.map_or(String::new(), |c| format!(" y dio fama a {c}"));
        let (title, text) = match feat {
            Feat::Invention => (
                format!("{} de {who}", crate::names::capitalize(&detail)),
                format!(
                    "En el año {year}, {doer} inventó {detail}. El invento cambió la vida de su tiempo{fame}."
                ),
            ),
            Feat::Treatise => (
                format!("El tratado de {who}"),
                format!(
                    "En el año {year}, {doer} escribió {detail}, que se copió y se estudió durante siglos."
                ),
            ),
            Feat::Expedition if self.age() == Age::Fantasy => (
                format!("La expedición de {who}"),
                format!(
                    "En el año {year}, {doer} se adentró en las ruinas olvidadas de {detail} y volvió con tesoros y secretos de una era perdida."
                ),
            ),
            Feat::Expedition => (
                format!("La expedición de {who}"),
                format!(
                    "En el año {year}, {doer} encabezó una expedición que recorrió y cartografió {detail}{fame}."
                ),
            ),
            Feat::Artifact => (
                crate::names::capitalize(&detail),
                format!(
                    "En el año {year}, {doer} forjó {detail}, un artefacto de poder. Desde entonces pasó de mano en mano como una reliquia."
                ),
            ),
            Feat::Grimoire => (
                crate::names::capitalize(&detail),
                format!(
                    "En el año {year}, {doer} descifró {detail}. Muchos advirtieron que ciertos saberes debían seguir enterrados."
                ),
            ),
            Feat::ArtificialMind => (
                format!("La creación de {detail}"),
                format!(
                    "En el año {year}, {doer} dio vida a {detail}, una inteligencia artificial capaz de aprender sola."
                ),
            ),
            Feat::Implant => (
                format!("El implante de {who}"),
                format!(
                    "En el año {year}, {doer} desarrolló {detail}. Pronto todos los que podían pagarlo lo llevaban."
                ),
            ),
            Feat::Heist => {
                let victim = target.map_or("un rival", |t| self.country(t));
                (
                    format!("El golpe de {who} contra {victim}"),
                    format!(
                        "En el año {year}, {doer} burló los cortafuegos de {victim} y robó sus datos secretos. Las relaciones entre ambos se envenenaron."
                    ),
                )
            }
            Feat::Rescue => unreachable!(),
        };
        (title, text)
    }

    // ------------------------------------------------------------ de conjunto

    /// Dos países que se enfrentaron en varias guerras.
    fn rivalries(&self, stories: &mut Vec<Story>) {
        let mut pairs: BTreeMap<(CountryId, CountryId), Vec<StoryId>> = BTreeMap::new();
        for s in stories.iter().filter(|s| s.kind == StoryKind::War) {
            let EventKind::WarDeclared { war } = self.event(s.events[0]).kind else {
                continue;
            };
            let w = self.world.war(war);
            let key = (w.attacker.min(w.defender), w.attacker.max(w.defender));
            pairs.entry(key).or_default().push(s.id);
        }
        for ((a, b), wars) in pairs {
            if wars.len() < 2 {
                continue;
            }
            let id = StoryId(stories.len() as u32);
            let mut rng = self.rng(&format!("rivalry-{}-{}", a.0, b.0));
            let (na, nb) = (self.country(a), self.country(b));
            let title = [
                format!("Las guerras entre {na} y {nb}"),
                format!("La Enemistad de {na} y {nb}"),
                format!("El Largo Conflicto de {na} y {nb}"),
            ]
            .choose(&mut rng)
            .unwrap()
            .clone();
            let start = stories[wars[0].index()].start;
            let last = &stories[wars[wars.len() - 1].index()];
            let end = wars
                .iter()
                .map(|w| stories[w.index()].end)
                .max()
                .unwrap_or(start);
            let ongoing = last.ongoing;
            let names: Vec<String> = wars
                .iter()
                .map(|w| {
                    let s = &stories[w.index()];
                    format!("la {} ({})", s.title, s.start)
                })
                .collect();
            let (mut wins_a, mut wins_b, mut draws) = (0, 0, 0);
            for s in &wars {
                if let EventKind::WarDeclared { war } =
                    self.event(stories[s.index()].events[0]).kind
                {
                    match self.world.war(war).winner {
                        Some(c) if c == a => wins_a += 1,
                        Some(_) => wins_b += 1,
                        None if self.world.war(war).ended.is_some() => draws += 1,
                        None => {}
                    }
                }
            }
            let mut text = format!(
                "Durante {}, {na} y {nb} se enfrentaron en {} guerras: {}.",
                span(end - start),
                wars.len(),
                list(&names)
            );
            let wins = |n: usize| match n {
                0 => "ninguna".to_string(),
                1 => "una".to_string(),
                n => format!("{n}"),
            };
            text += &format!(" {na} ganó {}, {nb} {}", wins(wins_a), wins(wins_b));
            match draws {
                0 => {}
                1 => text += ", y una acabó sin vencedor",
                n => text += &format!(", y {n} acabaron sin vencedor"),
            }
            text += ".";
            text += if ongoing {
                " La enemistad sigue viva y las armas no han callado."
            } else {
                " Cada paz fue solo una tregua hasta la siguiente."
            };
            let mut events: Vec<EventId> = wars
                .iter()
                .flat_map(|w| stories[w.index()].events.iter().copied())
                .collect();
            events.sort();
            for w in &wars {
                stories[w.index()].part_of = Some(id);
            }
            stories.push(Story {
                id,
                kind: StoryKind::Rivalry,
                title,
                start,
                end,
                ongoing,
                subjects: self.subjects(&events),
                events,
                summary: text,
                causes: Vec::new(),
                consequences: Vec::new(),
                chapters: wars,
                part_of: None,
            });
        }
    }

    /// Tres o más gobernantes seguidos de padres a hijos.
    fn dynasties(&self, stories: &mut Vec<Story>) {
        let world = self.world;
        for country in &world.countries {
            let crowns: Vec<&Event> = world
                .events
                .iter()
                .filter(|e| matches!(e.kind, EventKind::RulerCrowned { country: c, .. } if c == country.id))
                .collect();
            let person = |e: &Event| match e.kind {
                EventKind::RulerCrowned { person, .. } => person,
                _ => unreachable!(),
            };
            let mut i = 0;
            while i < crowns.len() {
                let mut j = i + 1;
                while j < crowns.len()
                    && world.person(person(crowns[j])).parent == Some(person(crowns[j - 1]))
                {
                    j += 1;
                }
                if j - i >= 3 {
                    self.dynasty(stories, country, &crowns[i..j], crowns.get(j).copied());
                }
                i = j;
            }
        }
    }

    fn dynasty(
        &self,
        stories: &mut Vec<Story>,
        country: &Country,
        run: &[&Event],
        next: Option<&Event>,
    ) {
        let world = self.world;
        let rulers: Vec<PersonId> = run
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::RulerCrowned { person, .. } => Some(person),
                _ => None,
            })
            .collect();
        let founder = world.person(rulers[0]);
        let house_word = self.age().house();
        let house = founder.name.split(' ').next_back().unwrap_or(&founder.name);
        let last = *rulers.last().unwrap();
        let start = run[0].year;
        let mut events: Vec<EventId> = run.iter().map(|e| e.id).collect();
        for e in &world.events {
            if let EventKind::PersonDied { person, .. }
            | EventKind::Assassinated { victim: person, .. } = e.kind
            {
                if rulers.contains(&person) {
                    events.push(e.id);
                }
            }
        }
        let names: Vec<String> = run
            .iter()
            .map(|e| match e.kind {
                EventKind::RulerCrowned { person, .. } => {
                    format!("{} ({})", self.person(person), e.year)
                }
                _ => unreachable!(),
            })
            .collect();
        let (end, ongoing, ending) = match next {
            Some(n) => {
                events.push(n.id);
                let EventKind::RulerCrowned { person: heir, .. } = n.kind else {
                    unreachable!()
                };
                let usurped = n.cause.is_some_and(|c| {
                    matches!(self.event(c).kind, EventKind::Assassinated { plotter, .. } if plotter == heir)
                });
                let consort = world.person(last).spouse == Some(heir);
                let text = if usurped {
                    format!(
                        " La dinastía terminó en el año {} cuando {} asesinó a {} y se hizo con el trono.",
                        n.year,
                        self.person(heir),
                        self.person(last)
                    )
                } else if consort {
                    format!(
                        " A la muerte de {}, sin hijos que lo sucedieran, el trono pasó a su consorte {} en el año {}.",
                        self.person(last),
                        self.person(heir),
                        n.year
                    )
                } else {
                    format!(
                        " La línea directa se extinguió con {}, y en el año {} el gobierno pasó a {}.",
                        self.person(last),
                        n.year,
                        self.person(heir)
                    )
                };
                (n.year, false, text)
            }
            None => match country.fell {
                Some(y) => (
                    y,
                    false,
                    format!(" La dinastía cayó con {} en el año {y}.", country.name),
                ),
                None => (
                    world.year,
                    true,
                    format!(" {house_word} {house} todavía gobierna {}.", country.name),
                ),
            },
        };
        let mut text = format!(
            "{house_word} {house} gobernó {} durante {}, de padres a hijos: {}.",
            country.name,
            span(end - start),
            list(&names)
        );
        text += &format!(
            " La fundó {} al asumir el gobierno en el año {start}.",
            founder.name
        );
        text += &ending;
        events.sort();
        events.dedup();
        let id = StoryId(stories.len() as u32);
        stories.push(Story {
            id,
            kind: StoryKind::Dynasty,
            title: format!("{house_word} {house} de {}", country.name),
            start,
            end,
            ongoing,
            subjects: self.subjects(&events),
            events,
            summary: text,
            causes: Vec::new(),
            consequences: Vec::new(),
            chapters: Vec::new(),
            part_of: None,
        });
    }

    /// Medio siglo o más sin guerras, pestes, calamidades, rebeliones ni
    /// magnicidios.
    fn golden_ages(&self, stories: &mut Vec<Story>) {
        const MIN_YEARS: i32 = 50;
        let world = self.world;
        for country in &world.countries {
            let c = country.id;
            let rulers = world.rulers_of(c);
            // Años turbulentos: (inicio, fin, evento que los abre).
            let mut troubles: Vec<(Year, Year, EventId)> = Vec::new();
            for e in &world.events {
                let trouble = match e.kind {
                    EventKind::WarDeclared { war } => {
                        let w = world.war(war);
                        (w.attacker == c || w.defender == c)
                            .then(|| (e.year, w.ended.unwrap_or(world.year)))
                    }
                    EventKind::Plague { country: p, .. }
                    | EventKind::Calamity { country: p, .. }
                        if p == c =>
                    {
                        Some((e.year, e.year))
                    }
                    EventKind::Independence { from, .. } if from == c => Some((e.year, e.year)),
                    EventKind::Assassinated { victim, .. }
                        if rulers.iter().any(|&(_, r)| r == victim) =>
                    {
                        Some((e.year, e.year))
                    }
                    _ => None,
                };
                if let Some((a, b)) = trouble {
                    troubles.push((a, b, e.id));
                }
            }
            troubles.sort();
            let finish = country.fell.unwrap_or(world.year);
            // Una edad de oro sigue a tiempos difíciles: la calma desde la
            // fundación no cuenta.
            let mut calm_from = Year::MAX;
            let mut best: Option<(Year, Year, Option<EventId>)> = None;
            let mut consider = |from: Year, to: Year, ended_by: Option<EventId>| {
                if to - from + 1 >= MIN_YEARS && best.is_none_or(|(a, b, _)| to - from > b - a) {
                    best = Some((from, to, ended_by));
                }
            };
            for &(a, b, id) in &troubles {
                if a > calm_from {
                    consider(calm_from, (a - 1).min(finish), Some(id));
                }
                calm_from = if calm_from == Year::MAX {
                    b + 1
                } else {
                    calm_from.max(b + 1)
                };
            }
            if calm_from <= finish {
                consider(calm_from, finish, None);
            }
            let Some((from, to, ended_by)) = best else {
                continue;
            };
            self.golden_age(stories, country, from, to, ended_by);
        }
    }

    fn golden_age(
        &self,
        stories: &mut Vec<Story>,
        country: &Country,
        from: Year,
        to: Year,
        ended_by: Option<EventId>,
    ) {
        let world = self.world;
        let c = country.id;
        let ours = |s: &Subject| match *s {
            Subject::Country(x) => x == c,
            Subject::Locality(l) => world.locality(l).country == Some(c),
            _ => false,
        };
        let events: Vec<EventId> = world
            .events
            .iter()
            .filter(|e| (from..=to).contains(&e.year))
            .filter(|e| {
                matches!(
                    e.kind,
                    EventKind::LocalityFounded { .. }
                        | EventKind::LocalityGrew { .. }
                        | EventKind::LocalityAnnexed { .. }
                        | EventKind::AllianceFormed { .. }
                        | EventKind::RulerCrowned { .. }
                )
            })
            .filter(|e| e.kind.subjects().iter().any(ours))
            .map(|e| e.id)
            .collect();
        if events.len() < 3 {
            return;
        }
        let count = |f: &dyn Fn(&EventKind) -> bool| {
            events.iter().filter(|&&e| f(&self.event(e).kind)).count()
        };
        let grew = count(&|k| matches!(k, EventKind::LocalityGrew { .. }));
        let founded = count(&|k| matches!(k, EventKind::LocalityFounded { .. }));
        let annexed = count(&|k| matches!(k, EventKind::LocalityAnnexed { .. }));
        let mut rulers: Vec<PersonId> = self.ruler_at(c, from).into_iter().collect();
        for &e in &events {
            if let EventKind::RulerCrowned { person, .. } = self.event(e).kind {
                if !rulers.contains(&person) {
                    rulers.push(person);
                }
            }
        }
        let title = match rulers.as_slice() {
            [one] => format!("La Paz de {}", self.person(*one)),
            _ if grew + founded >= 3 => format!("La edad de oro de {}", country.name),
            _ => format!("La Larga Paz de {}", country.name),
        };
        let age = self.age();
        let mut text = format!(
            "Entre los años {from} y {to}, {} vivió {} sin guerras, {}s, calamidades ni rebeliones.",
            country.name,
            span(to - from + 1),
            age.plague()
        );
        let village = LocalityKind::Village.noun(age);
        let (town, city) = (LocalityKind::Town.noun(age), LocalityKind::City.noun(age));
        let lower = |s: &str| s.to_lowercase();
        let mut growth = Vec::new();
        match founded {
            0 => {}
            1 => growth.push(format!("se fundó {}", lower(&village.a()))),
            n => growth.push(format!("se fundaron {n} {}", village.many)),
        }
        match annexed {
            0 => {}
            1 => growth.push(format!("se le unió {} independiente", lower(&village.a()))),
            n => growth.push(format!("se le unieron {n} {} independientes", village.many)),
        }
        match grew {
            0 => {}
            1 => growth.push(format!(
                "una localidad creció hasta convertirse en {} o {}",
                lower(town.one),
                lower(city.one)
            )),
            n => growth.push(format!(
                "{n} localidades crecieron hasta convertirse en {} o {}",
                town.many, city.many
            )),
        }
        if !growth.is_empty() {
            let mut sentence = list(&growth);
            if let Some(first) = sentence.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            text += &format!(" {sentence}.");
        }
        let allies: Vec<String> = events
            .iter()
            .filter_map(|&e| match self.event(e).kind {
                EventKind::AllianceFormed { a, b } => {
                    Some(self.country(if a == c { b } else { a }).to_string())
                }
                _ => None,
            })
            .collect();
        if !allies.is_empty() {
            text += &format!(" Selló alianzas con {}.", list(&allies));
        }
        let names: Vec<String> = rulers.iter().map(|&r| self.person(r).to_string()).collect();
        match names.len() {
            0 => {}
            1 => text += &format!(" Gobernó todo ese tiempo {}.", names[0]),
            _ => text += &format!(" Gobernaron en esos años {}.", list(&names)),
        }
        let ongoing = ended_by.is_none() && country.exists();
        match ended_by {
            Some(e) => {
                let e = self.event(e);
                text += &format!(
                    " La calma terminó en el año {}: {}.",
                    e.year,
                    world.describe(e)
                );
            }
            None if ongoing => text += " La paz dura hasta hoy.",
            None => {}
        }
        let id = StoryId(stories.len() as u32);
        let mut subjects = vec![Subject::Country(c)];
        for s in self.subjects(&events) {
            if !subjects.contains(&s) {
                subjects.push(s);
            }
        }
        stories.push(Story {
            id,
            kind: StoryKind::GoldenAge,
            title,
            start: from,
            end: to,
            ongoing,
            subjects,
            events,
            summary: text,
            causes: Vec::new(),
            consequences: Vec::new(),
            chapters: Vec::new(),
            part_of: None,
        });
    }
}
