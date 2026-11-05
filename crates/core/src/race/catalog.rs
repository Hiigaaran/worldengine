//! Catálogo de arquetipos: la materia prima con la que se generan las razas de
//! cada mundo. Cada edad tiene los suyos; cada mundo toma unos cuantos y, de
//! cada uno, solo una parte de sus rasgos, sonidos y costumbres, así que dos
//! mundos con elfos no tienen los mismos elfos.
//!
//! Los rasgos visibles van en español y en inglés `(es, en)`, y los colores
//! con su valor hexadecimal `(es, en, #rrggbb)`: el español para leer, el
//! inglés para los prompts de IA generativa y el color para los retratos.

use crate::model::Biome;

use super::{Marks, NameStyle, RaceKind};

pub(crate) type T = (&'static str, &'static str);
pub(crate) type C = (&'static str, &'static str, &'static str);

pub(crate) struct Sounds {
    pub onsets: &'static [&'static str],
    pub vowels: &'static [&'static str],
    pub codas: &'static [&'static str],
    pub style: NameStyle,
}

/// Costumbres de un arquetipo, una lista de opciones por aspecto.
pub(crate) struct Customs {
    pub rites: &'static [&'static str],
    pub marriage: &'static [&'static str],
    pub funerals: &'static [&'static str],
    pub food: &'static [&'static str],
    pub beliefs: &'static [&'static str],
    pub taboos: &'static [&'static str],
    pub writing: &'static [&'static str],
}

pub(crate) struct Archetype {
    pub kind: RaceKind,
    /// Biomas donde prospera; vacío si se adapta a cualquiera.
    pub biomes: &'static [Biome],
    pub height: (u16, u16),
    pub lifespan: u16,
    pub builds: &'static [T],
    /// Vacío en los pueblos realistas: su piel sale del clima de su tierra.
    pub skins: &'static [C],
    pub hair: &'static [C],
    pub eyes: &'static [T],
    pub features: &'static [T],
    pub attire: &'static [T],
    pub marks: Marks,
    pub temperament: &'static [&'static str],
    pub values: &'static [&'static str],
    pub sounds: Sounds,
    pub naming: &'static [&'static str],
    /// Lo que desea un saludo, en español.
    pub greetings: &'static [&'static str],
    pub customs: Customs,
}

const NONE: Marks = Marks {
    pointed_ears: false,
    tusks: false,
    scales: false,
    horns: false,
    cyber_eye: false,
    synthetic: false,
    beard_all: false,
    beardless: false,
    hairless: false,
};

// --- Pueblos realistas: todos humanos; su piel, pelo y ojos dependen del clima.

/// Tonos de piel del más oscuro al más claro.
pub(crate) const REAL_SKINS: [C; 7] = [
    ("muy oscura", "very dark brown", "#4a2c1d"),
    ("oscura", "dark brown", "#5f3b26"),
    ("morena", "brown", "#8a5a3b"),
    ("cobriza", "copper-toned", "#b07b52"),
    ("aceitunada", "olive", "#c9a072"),
    ("clara", "fair", "#e8bf9a"),
    ("muy pálida", "very pale", "#f3d5bd"),
];
pub(crate) const REAL_HAIR: [C; 7] = [
    ("pelo negro", "black", "#161616"),
    ("pelo negro azabache", "jet-black", "#0e0e12"),
    ("pelo castaño oscuro", "dark brown", "#2b1d14"),
    ("pelo castaño", "brown", "#5a3a22"),
    ("pelo cobrizo", "auburn", "#a8662f"),
    ("pelo pelirrojo", "red", "#b4462a"),
    ("pelo rubio", "blond", "#d8b25a"),
];
pub(crate) const REAL_EYES: [T; 6] = [
    ("negros", "black"),
    ("castaño oscuro", "dark brown"),
    ("castaños", "brown"),
    ("color avellana", "hazel"),
    ("verdes", "green"),
    ("azules", "blue"),
];

const REAL_BUILDS: &[T] = &[
    ("robusta", "stocky"),
    ("delgada", "lean"),
    ("atlética", "athletic"),
    ("nervuda", "wiry"),
    ("corpulenta", "heavyset"),
];

pub(crate) const REALISTIC: &[Archetype] = &[
    Archetype {
        kind: RaceKind::RiverFolk,
        biomes: &[Biome::Grassland, Biome::Savanna, Biome::Swamp],
        height: (155, 180),
        lifespan: 70,
        builds: REAL_BUILDS,
        skins: &[],
        hair: &[],
        eyes: &[],
        features: &[
            ("ojos delineados con kohl", "kohl-lined eyes"),
            ("manos callosas de labrar", "calloused farmer's hands"),
            ("un collar de cuentas de barro cocido", "a necklace of fired-clay beads"),
            ("la piel curtida por el sol", "sun-weathered skin"),
        ],
        attire: &[
            ("túnicas de lino blanco ceñidas con un fajín", "white linen tunics tied with a sash"),
            ("faldas plisadas y sandalias de papiro", "pleated kilts and papyrus sandals"),
            ("mantos de lana teñidos de ocre", "ochre-dyed wool cloaks"),
        ],
        marks: NONE,
        temperament: &["paciencia", "laboriosidad", "piedad", "prudencia", "hospitalidad", "sentido del orden"],
        values: &["la cosecha compartida", "la memoria de los antepasados", "el respeto a la ley escrita", "la obediencia a los ancianos"],
        sounds: Sounds {
            onsets: &["b", "d", "m", "n", "r", "s", "t", "l", "h", "k", "sh", "", ""],
            vowels: &["a", "e", "i", "o", "u", "a", "aa", "e"],
            codas: &["", "", "n", "t", "s", "m", "r"],
            style: NameStyle::Plain,
        },
        naming: &[
            "El nombre de pila lo elige la abuela; el apellido es el del canal o la acequia de la familia.",
            "Cada niño recibe el nombre de un antepasado y el apellido del linaje materno.",
        ],
        greetings: &["que el río te sea generoso", "que tu granero no se vacíe", "paz en tu casa"],
        customs: Customs {
            rites: &[
                "A los catorce años, cada joven siembra y cosecha solo su primer campo.",
                "Los jóvenes pasan una noche a orillas del río esperando una señal de los dioses.",
                "Se marca la mayoría de edad con un tatuaje de juncos en la muñeca.",
            ],
            marriage: &[
                "Las familias intercambian semillas y tierras; la boda se celebra en la crecida.",
                "Los novios beben del mismo cuenco de agua del río ante el templo.",
                "El matrimonio es un contrato escrito que firman testigos de ambas familias.",
            ],
            funerals: &[
                "Los muertos se embalsaman y se entierran con comida para el viaje.",
                "Los cuerpos se entregan a la corriente en barcas de juncos.",
                "Se entierra a los muertos bajo el suelo de la casa familiar.",
            ],
            food: &[
                "Pan de cebada, cerveza espesa y pescado de río secado al sol.",
                "Gachas de mijo, dátiles y lentejas guisadas.",
                "Arroz con legumbres y patos cebados para las fiestas.",
            ],
            beliefs: &[
                "Adoran al río como a un dios que da y quita la vida.",
                "Creen que el sol muere cada noche y que las plegarias lo hacen renacer.",
                "Un panteón de dioses de la cosecha, cada uno con su templo y sus sacerdotes.",
            ],
            taboos: &[
                "Ensuciar el agua del río se castiga con el destierro.",
                "Nadie mueve un mojón de linde sin un sacerdote presente.",
                "Es impuro comer la carne de los animales de tiro.",
            ],
            writing: &[
                "Escriben con cañas sobre tablillas de arcilla.",
                "Usan jeroglíficos pintados en papiro y piedra.",
                "Llevan las cuentas con cordeles anudados.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Steppe,
        biomes: &[Biome::Grassland, Biome::Shrubland, Biome::Savanna],
        height: (158, 182),
        lifespan: 60,
        builds: REAL_BUILDS,
        skins: &[],
        hair: &[],
        eyes: &[],
        features: &[
            ("piernas arqueadas de toda una vida a caballo", "bowed legs from a life on horseback"),
            ("la cara curtida por el viento", "wind-burned face"),
            ("trenzas adornadas con cuentas de hueso", "braids decorated with bone beads"),
            ("cicatrices rituales en las mejillas", "ritual scars on the cheeks"),
        ],
        attire: &[
            ("caftanes de fieltro forrados de piel", "fur-lined felt kaftans"),
            ("botas altas de cuero y gorros puntiagudos", "tall leather boots and pointed hats"),
            ("capas de lana con bordados geométricos", "wool capes with geometric embroidery"),
        ],
        marks: NONE,
        temperament: &["audacia", "franqueza", "lealtad", "sentido del honor", "orgullo", "dureza"],
        values: &["la libertad de moverse", "la hospitalidad con el viajero", "la lealtad al clan", "el valor en la batalla"],
        sounds: Sounds {
            onsets: &["b", "ch", "d", "g", "j", "k", "m", "n", "s", "t", "ts", "kh", "", ""],
            vowels: &["a", "e", "i", "o", "u", "ö", "ü", "a"],
            codas: &["", "n", "r", "l", "g", "t", "an", "ai"],
            style: NameStyle::Plain,
        },
        naming: &[
            "El niño recibe un nombre provisional para engañar a los malos espíritus y el verdadero al cumplir siete años.",
            "El apellido es el del clan; el nombre suele ser el de un animal o una virtud.",
        ],
        greetings: &["que tus caballos estén sanos", "que el cielo azul te guarde", "que tu yurta esté caliente"],
        customs: Customs {
            rites: &[
                "Los jóvenes deben domar solos su primer caballo.",
                "La mayoría de edad llega al cazar el primer lobo.",
                "A los doce años, cada niño recibe su arco y su propia montura.",
            ],
            marriage: &[
                "El novio rapta a la novia con su consentimiento y paga un precio en caballos.",
                "Las bodas sellan alianzas entre clanes y duran tres días de carreras y banquetes.",
                "Los novios atan sus cintos y saltan juntos sobre una hoguera.",
            ],
            funerals: &[
                "Los muertos se entierran en túmulos con su caballo y sus armas.",
                "Los cuerpos se dejan en la estepa para que el cielo los reclame.",
                "Se levantan estatuas de piedra que miran al este sobre las tumbas.",
            ],
            food: &[
                "Leche de yegua fermentada, carne seca y queso duro.",
                "Cordero hervido, caldo de huesos y té con mantequilla.",
                "Tortas de harina fritas en grasa y carne de caballo ahumada.",
            ],
            beliefs: &[
                "Veneran al cielo eterno y a los espíritus de los antepasados.",
                "Sus chamanes hablan con los espíritus al son del tambor.",
                "Creen que el alma vive en el aliento y que el viento la lleva lejos.",
            ],
            taboos: &[
                "Pisar el umbral de una yurta trae desgracia a la familia.",
                "Derramar leche en el suelo es una ofensa al cielo.",
                "Nadie corta la crin de un caballo sin pedirle permiso.",
            ],
            writing: &[
                "Graban runas angulosas en piedras y huesos.",
                "Escriben en columnas verticales con tinta de hollín.",
                "Apenas escriben: guardan la historia en largos cantares.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Seafarers,
        biomes: &[Biome::Forest, Biome::Grassland, Biome::Hills],
        height: (160, 188),
        lifespan: 65,
        builds: REAL_BUILDS,
        skins: &[],
        hair: &[],
        eyes: &[],
        features: &[
            ("manos llenas de cicatrices de cabos", "rope-scarred hands"),
            ("aros de oro en las orejas", "gold earrings"),
            ("tatuajes de olas y peces en los brazos", "wave and fish tattoos on the arms"),
            ("la mirada entornada de quien vigila el horizonte", "squinting eyes used to watching the horizon"),
        ],
        attire: &[
            ("camisas de lino y chalecos de piel de foca", "linen shirts and sealskin vests"),
            ("capas enceradas contra la lluvia", "waxed rain cloaks"),
            ("pantalones anchos y fajas de colores", "loose trousers and colorful sashes"),
        ],
        marks: NONE,
        temperament: &["osadía", "curiosidad", "superstición", "franqueza", "espíritu comerciante", "terquedad"],
        values: &["la palabra dada en un trato", "la fortuna del viajero", "la fraternidad de la tripulación", "la riqueza ganada en el mar"],
        sounds: Sounds {
            onsets: &["b", "d", "f", "h", "k", "l", "m", "n", "r", "s", "t", "v", "sk", "st", ""],
            vowels: &["a", "e", "i", "o", "u", "ei", "au", "a"],
            codas: &["", "", "r", "n", "d", "k", "ll", "st"],
            style: NameStyle::Plain,
        },
        naming: &[
            "Se lleva el nombre del barco en que se nació como apellido.",
            "El apellido indica el padre o la madre: «hijo de», «hija de», abreviado en la última sílaba.",
        ],
        greetings: &["buen viento en tus velas", "que el mar te devuelva", "que tu red vuelva llena"],
        customs: Customs {
            rites: &[
                "Cada joven debe cruzar a nado la bahía en invierno.",
                "La mayoría de edad llega con el primer viaje sin ver la costa.",
                "Los aprendices se ganan el tatuaje de su primera ola al volver de alta mar.",
            ],
            marriage: &[
                "Los novios se casan en la proa de un barco y lanzan al mar una moneda.",
                "La novia teje una red y el novio construye un bote; juntos forman un hogar.",
                "Las bodas se celebran al volver la flota, todas el mismo día.",
            ],
            funerals: &[
                "Los muertos zarpan en barcas incendiadas al anochecer.",
                "Se entierra a los marinos con un remo clavado sobre la tumba.",
                "Los ahogados se recuerdan con piedras talladas frente al puerto.",
            ],
            food: &[
                "Bacalao salado, pan negro y cerveza amarga.",
                "Guisos de mariscos, algas y vino especiado.",
                "Pescado ahumado, gachas de avena y leche cuajada.",
            ],
            beliefs: &[
                "Temen y veneran a un dios del mar caprichoso al que ofrecen parte de cada pesca.",
                "Creen que los ahogados se vuelven gaviotas.",
                "Leen el destino en las mareas y en el vuelo de las aves.",
            ],
            taboos: &[
                "Silbar a bordo llama a la tormenta.",
                "Una mujer o un hombre que pierde su barco no puede volver a nombrarlo.",
                "Nadie zarpa el día de luna nueva.",
            ],
            writing: &[
                "Un alfabeto de trazos rectos, fácil de tallar en madera.",
                "Escriben en pergamino y guardan cartas de navegación como tesoros.",
                "Marcan sus mercancías con signos comerciales que todo puerto entiende.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Highlanders,
        biomes: &[Biome::Hills, Biome::Mountain, Biome::Taiga],
        height: (158, 184),
        lifespan: 65,
        builds: REAL_BUILDS,
        skins: &[],
        hair: &[],
        eyes: &[],
        features: &[
            ("pantorrillas fuertes de subir laderas", "strong calves from climbing slopes"),
            ("mejillas enrojecidas por el frío", "cold-reddened cheeks"),
            ("una nariz rota alguna vez", "a once-broken nose"),
            ("trenzas recogidas bajo un pañuelo", "braids tucked under a headscarf"),
        ],
        attire: &[
            ("mantas de lana a cuadros prendidas al hombro", "plaid wool blankets pinned at the shoulder"),
            ("chaquetas de piel de cabra y gorros de fieltro", "goatskin jackets and felt caps"),
            ("ponchos tejidos con franjas de colores vivos", "woven ponchos with bright stripes"),
        ],
        marks: NONE,
        temperament: &["terquedad", "lealtad", "orgullo", "resistencia", "desconfianza", "humor seco"],
        values: &["la independencia del valle", "la venganza de las ofensas", "la palabra del jefe", "la tierra de los padres"],
        sounds: Sounds {
            onsets: &["b", "c", "d", "g", "l", "m", "n", "r", "t", "br", "dr", "gw", "qu", ""],
            vowels: &["a", "e", "i", "o", "u", "ae", "ei", "a"],
            codas: &["", "", "n", "r", "ch", "dd", "th", "s"],
            style: NameStyle::Plain,
        },
        naming: &[
            "El apellido es el del valle; quien se muda lo cambia por el de su nuevo hogar.",
            "Los niños llevan el nombre del abuelo y un mote que se ganan con los años.",
        ],
        greetings: &["que tu fuego no se apague", "que la montaña te sostenga", "salud y rebaños"],
        customs: Customs {
            rites: &[
                "Los jóvenes suben solos a la cumbre más cercana y vuelven con una piedra.",
                "La mayoría de edad se gana en un torneo de lanzamiento de troncos.",
                "Cada joven pasa un verano pastoreando solo en los pastos altos.",
            ],
            marriage: &[
                "Los novios se dan las manos atadas con un cordón del tartán familiar.",
                "El novio debe trabajar un año en la casa de la novia.",
                "Las bodas unen clanes rivales y se celebran con gaitas y danzas de espadas.",
            ],
            funerals: &[
                "Se levantan túmulos de piedras: cada visitante añade una.",
                "Los muertos se velan tres noches con cantos de lamento.",
                "Se entierra a los muertos en cuevas de la montaña.",
            ],
            food: &[
                "Gachas de avena, queso de cabra y licor de cebada.",
                "Patatas, cordero estofado y pan de centeno.",
                "Maíz tostado, papas heladas y carne secada al frío.",
            ],
            beliefs: &[
                "Cada montaña tiene su espíritu, al que se ofrece hojas y licor.",
                "Creen en hadas traviesas que viven bajo las colinas.",
                "Veneran a los antepasados con santuarios en cada casa.",
            ],
            taboos: &[
                "Romper la hospitalidad dada bajo un techo es el peor crimen.",
                "No se señala la cumbre sagrada con el dedo.",
                "Nadie pronuncia el nombre de un muerto durante un año.",
            ],
            writing: &[
                "Tallan inscripciones de muescas en el borde de las piedras.",
                "Los bardos memorizan genealogías enteras; la escritura es para los forasteros.",
                "Usan un alfabeto prestado de los pueblos del llano.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Woodland,
        biomes: &[Biome::Forest, Biome::Jungle, Biome::Swamp],
        height: (152, 180),
        lifespan: 60,
        builds: REAL_BUILDS,
        skins: &[],
        hair: &[],
        eyes: &[],
        features: &[
            ("pinturas de arcilla en la cara", "clay paint on the face"),
            ("un collar de dientes de jabalí", "a boar-tusk necklace"),
            ("pies descalzos y endurecidos", "hardened bare feet"),
            ("plumas trenzadas en el pelo", "feathers braided into the hair"),
        ],
        attire: &[
            ("ropas de corteza batida y fibras vegetales", "garments of beaten bark and plant fibers"),
            ("túnicas de lino teñidas de verde y pardo", "linen tunics dyed green and brown"),
            ("capas de piel de oso y cinturones de cuero", "bearskin capes and leather belts"),
        ],
        marks: NONE,
        temperament: &["sigilo", "paciencia", "reserva", "astucia", "espíritu comunitario", "respeto por la naturaleza"],
        values: &["el equilibrio con el bosque", "el silencio", "la caza justa", "la sabiduría de las abuelas"],
        sounds: Sounds {
            onsets: &["b", "v", "d", "z", "k", "l", "m", "n", "r", "s", "t", "br", "dr", "sv", "y", ""],
            vowels: &["a", "e", "i", "o", "u", "ia", "y", "o"],
            codas: &["", "", "n", "v", "sk", "k", "r", "sh"],
            style: NameStyle::Plain,
        },
        naming: &[
            "Cada persona tiene un nombre secreto y otro de uso diario; el secreto solo lo sabe la madre.",
            "El apellido es el árbol bajo el que se fundó la familia.",
        ],
        greetings: &["que el bosque te esconda", "que la caza te honre", "camina en silencio"],
        customs: Customs {
            rites: &[
                "Los jóvenes viven una luna solos en el bosque y vuelven con un nombre nuevo.",
                "La mayoría de edad se marca plantando un árbol que crecerá con la persona.",
                "Los cazadores novatos deben seguir un rastro durante tres días sin perderlo.",
            ],
            marriage: &[
                "Los novios intercambian ramas de su árbol familiar y las trenzan juntas.",
                "La pareja construye su cabaña antes de la boda, con ayuda de toda la aldea.",
                "Las bodas se celebran en el claro sagrado en la noche más corta del año.",
            ],
            funerals: &[
                "Los muertos se colocan en plataformas en las copas de los árboles.",
                "Se entierra a los muertos bajo un roble joven que se planta sobre ellos.",
                "Los cuerpos se queman y las cenizas se esparcen en el bosque.",
            ],
            food: &[
                "Venado asado, setas, bayas y miel silvestre.",
                "Tortas de bellota, pescado de río y raíces asadas.",
                "Mandioca, frutos de palma y carne de caza ahumada.",
            ],
            beliefs: &[
                "Cada árbol viejo tiene un espíritu que recuerda todo lo que ve.",
                "Veneran a una diosa madre del bosque que castiga a los codiciosos.",
                "Creen que los muertos renacen como animales del bosque.",
            ],
            taboos: &[
                "Talar un árbol sin pedirle perdón trae enfermedad.",
                "Nadie caza a una hembra preñada.",
                "Es impuro silbar de noche entre los árboles.",
            ],
            writing: &[
                "Tallan signos en la corteza para marcar caminos y avisos.",
                "No escriben: anudan cuerdas y pintan pieles para recordar.",
                "Usan un alfabeto de trazos inspirados en las ramas.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Desert,
        biomes: &[Biome::Desert, Biome::Savanna, Biome::Shrubland],
        height: (160, 185),
        lifespan: 65,
        builds: REAL_BUILDS,
        skins: &[],
        hair: &[],
        eyes: &[],
        features: &[
            ("los ojos rodeados de arrugas de entornarlos al sol", "sun-squint wrinkles around the eyes"),
            ("tatuajes índigo en la barbilla", "indigo tattoos on the chin"),
            ("brazaletes de plata en las muñecas", "silver bracelets on the wrists"),
            ("las manos teñidas de henna", "henna-dyed hands"),
        ],
        attire: &[
            ("velos y turbantes de algodón índigo", "indigo cotton veils and turbans"),
            ("túnicas holgadas de lana blanca", "loose white wool robes"),
            ("mantos de pelo de camello y sandalias", "camel-hair mantles and sandals"),
        ],
        marks: NONE,
        temperament: &["hospitalidad", "resistencia", "orgullo", "paciencia", "elocuencia", "sentido del honor"],
        values: &["el agua compartida", "la hospitalidad sagrada", "la palabra del poeta", "la fidelidad a la tribu"],
        sounds: Sounds {
            onsets: &["b", "d", "f", "h", "j", "k", "kh", "m", "n", "q", "r", "s", "sh", "t", "z", ""],
            vowels: &["a", "i", "u", "a", "aa", "ou", "i"],
            codas: &["", "", "d", "m", "n", "r", "f", "sh"],
            style: NameStyle::Plain,
        },
        naming: &[
            "Se nombra a cada persona por su linaje hasta el tercer abuelo, aunque se usa solo el último.",
            "El apellido es el del pozo junto al que nació la familia.",
        ],
        greetings: &["que nunca te falte el agua", "la paz sea en tu tienda", "que tu sombra sea larga"],
        customs: Customs {
            rites: &[
                "Los jóvenes guían solos una caravana pequeña de un oasis a otro.",
                "La mayoría de edad se celebra con el primer velo o el primer turbante.",
                "Cada joven recita ante la tribu un poema compuesto por sí mismo.",
            ],
            marriage: &[
                "La familia del novio entrega camellos; la de la novia, una tienda tejida a mano.",
                "Las bodas duran siete días y terminan con una carrera de camellos.",
                "Los novios comparten dátiles y leche bajo un dosel de palmas.",
            ],
            funerals: &[
                "Se entierra a los muertos antes del anochecer, mirando hacia el amanecer.",
                "Los muertos descansan bajo montones de piedras blancas en las dunas.",
                "Se entierra a los muertos en el oasis, junto a una palmera nueva.",
            ],
            food: &[
                "Dátiles, leche de camella y pan cocido bajo la arena.",
                "Cuscús de sémola, cordero especiado y té con menta.",
                "Tortas de mijo, queso de cabra y café amargo.",
            ],
            beliefs: &[
                "Creen en un único dios del cielo y en genios que habitan las dunas.",
                "Veneran las estrellas que guían las caravanas.",
                "Creen que el viento del desierto lleva las almas de los muertos.",
            ],
            taboos: &[
                "Negar agua a un viajero es una ofensa imperdonable.",
                "Nadie descubre su rostro ante extraños.",
                "Es impuro contaminar un pozo, aunque sea del enemigo.",
            ],
            writing: &[
                "Una caligrafía curva que es también un arte.",
                "Escriben poemas en tablillas de madera que se lavan y se reutilizan.",
                "Usan signos sencillos grabados en las rocas de los caminos.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Northern,
        biomes: &[Biome::Taiga, Biome::Tundra, Biome::Glacier],
        height: (155, 182),
        lifespan: 60,
        builds: REAL_BUILDS,
        skins: &[],
        hair: &[],
        eyes: &[],
        features: &[
            ("la cara ancha y los pómulos altos", "broad face and high cheekbones"),
            ("la nariz y las mejillas marcadas por el frío", "frostbite-marked nose and cheeks"),
            ("líneas tatuadas en la barbilla", "tattooed lines on the chin"),
            ("amuletos de marfil colgados del cuello", "ivory amulets around the neck"),
        ],
        attire: &[
            ("parkas de piel de reno con capucha", "hooded reindeer-fur parkas"),
            ("botas forradas y guantes de piel de foca", "lined boots and sealskin gloves"),
            ("abrigos de lana bordados con hilo rojo", "wool coats embroidered with red thread"),
        ],
        marks: NONE,
        temperament: &["resistencia", "calma", "generosidad", "reserva", "ingenio", "paciencia"],
        values: &["compartir la caza", "sobrevivir al invierno juntos", "el respeto a los animales", "la palabra de los chamanes"],
        sounds: Sounds {
            onsets: &["k", "t", "n", "s", "m", "v", "j", "h", "p", "l", "", ""],
            vowels: &["a", "e", "i", "o", "u", "ä", "y", "aa"],
            codas: &["", "", "n", "t", "k", "s", "lk"],
            style: NameStyle::Plain,
        },
        naming: &[
            "Se nombra a los niños como al último pariente fallecido, que así vuelve a vivir.",
            "El apellido es el del rebaño de renos de la familia.",
        ],
        greetings: &["que el hielo te sostenga", "calor en tu tienda", "que la aurora te guíe"],
        customs: Customs {
            rites: &[
                "Los jóvenes deben pasar una noche de invierno solos en un refugio de nieve.",
                "La primera foca o el primer reno cazado se reparte entre toda la aldea.",
                "Cada joven recibe su cuchillo y su trineo al cumplir trece inviernos.",
            ],
            marriage: &[
                "Los novios pasan juntos un invierno de prueba antes de casarse.",
                "La boda se sella regalando renos y pieles a la familia del otro.",
                "Los novios encienden juntos la lámpara de grasa de su primera tienda.",
            ],
            funerals: &[
                "Los muertos se dejan en la tundra cubiertos de piedras, con sus herramientas.",
                "Se entierra a los muertos en el hielo, mirando a la aurora.",
                "Los cuerpos se queman en piras de madera de deriva.",
            ],
            food: &[
                "Carne de reno, grasa de foca y bayas del verano.",
                "Pescado crudo congelado, sopa de sangre y pan de corteza.",
                "Leche de reno, queso seco y caldo de hueso.",
            ],
            beliefs: &[
                "Sus chamanes viajan al mundo de los espíritus en trance.",
                "Creen que la aurora son las almas de los muertos jugando.",
                "Veneran a un espíritu del mar que guarda a los animales.",
            ],
            taboos: &[
                "Burlarse de una presa cazada ofende a su espíritu.",
                "No se cose ropa nueva durante la temporada de caza.",
                "Nadie apaga la lámpara de otro.",
            ],
            writing: &[
                "Pintan signos en tambores de piel de reno.",
                "Tallan pictogramas en marfil y hueso.",
                "No escriben: cuentan su historia en largos cantos de invierno.",
            ],
        },
    },
];

// --- Razas de fantasía.

const HUMAN_HAIR: &[C] = &[
    ("pelo castaño", "brown", "#5a3a22"),
    ("pelo negro", "black", "#161616"),
    ("pelo rubio", "blond", "#d8b25a"),
    ("pelo pelirrojo", "red", "#b4462a"),
    ("pelo castaño oscuro", "dark brown", "#2b1d14"),
];
const HUMAN_SKINS: &[C] = &[
    ("clara", "fair", "#e8bf9a"),
    ("morena", "brown", "#8a5a3b"),
    ("aceitunada", "olive", "#c9a072"),
    ("oscura", "dark brown", "#5f3b26"),
    ("cobriza", "copper-toned", "#b07b52"),
];
const HUMAN_EYES: &[T] = &[
    ("castaños", "brown"),
    ("verdes", "green"),
    ("azules", "blue"),
    ("grises", "grey"),
    ("negros", "black"),
];

pub(crate) const FANTASY: &[Archetype] = &[
    Archetype {
        kind: RaceKind::Human,
        biomes: &[],
        height: (158, 186),
        lifespan: 75,
        builds: &[("atlética", "athletic"), ("robusta", "sturdy"), ("delgada", "lean"), ("corpulenta", "heavyset")],
        skins: HUMAN_SKINS,
        hair: HUMAN_HAIR,
        eyes: HUMAN_EYES,
        features: &[
            ("una cicatriz de espada en el mentón", "a sword scar on the chin"),
            ("un medallón con el emblema de su casa", "a medallion bearing their house crest"),
            ("las manos manchadas de tinta", "ink-stained hands"),
            ("un anillo de sello en el dedo", "a signet ring"),
        ],
        attire: &[
            ("jubones de terciopelo y capas con broche", "velvet doublets and brooched cloaks"),
            ("cotas de malla bajo sobrevestes con blasón", "chainmail under heraldic surcoats"),
            ("ropas de lana teñidas y botas de cuero", "dyed wool clothes and leather boots"),
        ],
        marks: NONE,
        temperament: &["ambición", "adaptabilidad", "valentía", "impaciencia", "ingenio", "codicia"],
        values: &["la gloria del linaje", "la fe en los dioses", "el poder de la corona", "el oro y la fortuna"],
        sounds: Sounds {
            onsets: &["b", "c", "d", "g", "l", "m", "r", "s", "t", "v", "br", "al", "ed", "", ""],
            vowels: &["a", "e", "i", "o", "u", "a", "e"],
            codas: &["", "", "n", "r", "s", "l", "rd", "ric"],
            style: NameStyle::Plain,
        },
        naming: &[
            "Nombre de pila y apellido de la casa; los nobles añaden el de su señorío.",
            "Los plebeyos se apellidan por su oficio o su aldea; los nobles, por su casa.",
        ],
        greetings: &["que los dioses te guarden", "buena fortuna en tu camino", "salud y gloria"],
        customs: Customs {
            rites: &[
                "A los dieciséis años se jura lealtad a un señor o a un templo.",
                "Los escuderos velan sus armas en la capilla antes de ser armados caballeros.",
                "Cada joven parte un año de aprendiz a casa de un maestro de oficio.",
            ],
            marriage: &[
                "Los matrimonios nobles se pactan por alianzas; el pueblo se casa en la fiesta de la cosecha.",
                "Un sacerdote ata las manos de los novios con una cinta bendecida.",
                "Las bodas terminan con un torneo en honor de la novia o del novio.",
            ],
            funerals: &[
                "Se entierra a los muertos en criptas familiares bajo el templo.",
                "Los héroes arden en piras y sus cenizas se guardan en urnas.",
                "Los muertos se entierran con una moneda para el barquero.",
            ],
            food: &[
                "Pan de trigo, cerdo asado, queso y vino tinto.",
                "Estofado de venado, cerveza y pasteles de carne.",
                "Gachas, nabos, tocino y sidra en los días de mercado.",
            ],
            beliefs: &[
                "Adoran a un panteón de dioses con templos en cada ciudad.",
                "Siguen a un dios solar cuyos paladines persiguen a los nigromantes.",
                "Temen la magia, aunque sus reyes contratan magos de corte.",
            ],
            taboos: &[
                "La nigromancia se castiga con la hoguera.",
                "Romper un juramento ante los dioses deshonra a toda la familia.",
                "Nadie entra armado a un templo.",
            ],
            writing: &[
                "Un alfabeto común que sirve de lengua franca entre las razas.",
                "Los escribas copian crónicas en pergamino iluminado.",
                "Usan el alfabeto común; los magos, glifos secretos.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Elf,
        biomes: &[Biome::Forest, Biome::Jungle, Biome::Taiga],
        height: (170, 198),
        lifespan: 700,
        builds: &[("esbelta", "slender"), ("grácil", "graceful"), ("alta y delgada", "tall and willowy")],
        skins: &[
            ("pálida como la luna", "moon-pale", "#f2e6dc"),
            ("dorada", "golden", "#e3c9a8"),
            ("color avellana", "hazel-brown", "#a87c5a"),
            ("gris ceniza", "ash-grey", "#9a98a8"),
        ],
        hair: &[
            ("cabello plateado", "silver", "#e6e6f2"),
            ("cabello dorado", "golden", "#e0c060"),
            ("cabello negro", "raven-black", "#161616"),
            ("cabello cobrizo", "copper", "#a8662f"),
            ("cabello blanco", "white", "#f4f4f4"),
        ],
        eyes: &[
            ("verde esmeralda", "emerald-green"),
            ("violeta", "violet"),
            ("plateados", "silver"),
            ("color ámbar", "amber"),
            ("azul profundo", "deep blue"),
        ],
        features: &[
            ("unas orejas largas y puntiagudas", "long pointed ears"),
            ("unos rasgos finos y sin edad", "fine, ageless features"),
            ("unos ojos que brillan levemente en la oscuridad", "eyes that faintly glow in the dark"),
            ("una diadema de plata entre el cabello", "a silver circlet in the hair"),
        ],
        attire: &[
            ("túnicas de seda bordadas con hojas", "silk robes embroidered with leaves"),
            ("capas grises que se confunden con el bosque", "grey cloaks that blend into the forest"),
            ("armaduras ligeras de escamas labradas", "light, finely carved scale armor"),
        ],
        marks: Marks {
            pointed_ears: true,
            beardless: true,
            ..NONE
        },
        temperament: &["orgullo", "paciencia", "melancolía", "elegancia", "desdén por las razas jóvenes", "amor por la belleza"],
        values: &["la memoria de los siglos", "la armonía con la naturaleza", "el arte y la música", "el linaje puro"],
        sounds: Sounds {
            onsets: &["l", "n", "s", "th", "v", "f", "m", "r", "el", "gal", "c", "", ""],
            vowels: &["a", "e", "i", "ë", "ae", "ia", "ie", "ó"],
            codas: &["", "", "n", "l", "th", "s", "r", "nd", "wen"],
            style: NameStyle::Plain,
        },
        naming: &[
            "Tienen un nombre de infancia, otro de adultez y uno secreto que solo se dice al morir.",
            "El apellido es el de la estrella bajo la que nacieron; cambia si se exilian.",
        ],
        greetings: &["que las estrellas iluminen tu camino", "que tu canto no se olvide", "bajo hojas y luz"],
        customs: Customs {
            rites: &[
                "A los cien años, cada elfo compone un canto que lo presenta ante su linaje.",
                "Los jóvenes plantan un árbol de plata que vivirá tanto como ellos.",
                "La mayoría de edad llega tras un siglo de aprendizaje con un maestro de arte.",
            ],
            marriage: &[
                "Se casan una sola vez en su larga vida; la promesa se canta bajo la luna llena.",
                "Los novios intercambian anillos forjados con la plata de su árbol natal.",
                "Una boda élfica dura un año entero de fiestas y estaciones.",
            ],
            funerals: &[
                "Los muertos navegan hacia el oeste en barcas blancas.",
                "Se entierra a los muertos bajo árboles que florecen al año siguiente.",
                "Sus cuerpos se dejan en salones de cristal donde nunca se corrompen.",
            ],
            food: &[
                "Pan de viaje ligero, frutos del bosque y vino de flores.",
                "Néctar, setas silvestres y pescado de lagos claros.",
                "Tortas de miel, frutos secos y agua de manantial perfumada.",
            ],
            beliefs: &[
                "Veneran a las estrellas y a la luz que hubo antes del sol.",
                "Creen que la naturaleza entera canta y que la magia es su música.",
                "Esperan volver algún día a una tierra perdida al otro lado del mar.",
            ],
            taboos: &[
                "Cortar un árbol vivo es casi un asesinato.",
                "Nadie revela su nombre secreto a otra raza.",
                "Comer carne de criaturas que cantan está prohibido.",
            ],
            writing: &[
                "Una escritura curva y fluida, grabada en hojas de plata.",
                "Runas lunares que solo se leen bajo cierta luz.",
                "Caligrafía de trazos largos tejida en tapices.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Dwarf,
        biomes: &[Biome::Hills, Biome::Mountain, Biome::Tundra],
        height: (125, 150),
        lifespan: 300,
        builds: &[("fornida", "stout"), ("ancha y robusta", "broad and sturdy"), ("maciza", "barrel-chested")],
        skins: &[
            ("rubicunda", "ruddy", "#d9a184"),
            ("curtida por las forjas", "forge-tanned", "#b07b52"),
            ("pardo rojiza", "reddish-brown", "#8a5a3b"),
            ("gris como la piedra", "stone-grey", "#a39a92"),
        ],
        hair: &[
            ("pelo rojo fuego", "fiery red", "#9b2e1f"),
            ("pelo castaño", "brown", "#5a3a22"),
            ("pelo negro", "black", "#161616"),
            ("pelo gris acero", "steel-grey", "#8c8c8c"),
            ("pelo rubio cobrizo", "copper-blond", "#c08a3a"),
        ],
        eyes: &[
            ("negros como el carbón", "coal-black"),
            ("grises", "grey"),
            ("castaños", "brown"),
            ("azul hielo", "ice-blue"),
        ],
        features: &[
            ("una barba trenzada con anillos de metal", "a braided beard with metal rings"),
            ("una nariz ancha y prominente", "a broad, prominent nose"),
            ("unos brazos llenos de quemaduras de forja", "forge-burned forearms"),
            ("tatuajes de runas en la frente", "runic tattoos on the forehead"),
        ],
        attire: &[
            ("cotas de malla bajo mandiles de cuero", "chainmail under leather aprons"),
            ("yelmos con cuernos y capas de piel", "horned helmets and fur cloaks"),
            ("túnicas pesadas con hebillas de bronce", "heavy tunics with bronze buckles"),
        ],
        marks: Marks {
            beard_all: true,
            ..NONE
        },
        temperament: &["terquedad", "lealtad", "rencor", "laboriosidad", "orgullo", "desconfianza"],
        values: &["la obra bien hecha", "la memoria de las ofensas", "el oro y las gemas", "el honor del clan"],
        sounds: Sounds {
            onsets: &["b", "d", "g", "k", "th", "dr", "gr", "kh", "br", "thr", "dw", ""],
            vowels: &["a", "o", "u", "i", "a", "u", "e"],
            codas: &["", "n", "r", "k", "m", "rn", "nd", "rd", "dd", "in"],
            style: NameStyle::Plain,
        },
        naming: &[
            "Se presentan como «hijo de» o «hija de» y el nombre de su clan; su nombre verdadero lo graban solo en su tumba.",
            "El apellido es el del clan, y se antepone el nombre de la mina natal en ocasiones solemnes.",
        ],
        greetings: &["que tu forja nunca se enfríe", "que tu barba crezca larga", "piedra y honor"],
        customs: Customs {
            rites: &[
                "Cada joven forja su primera hacha y la presenta ante los ancianos del clan.",
                "La mayoría de edad llega a los cincuenta años, con la primera trenza de la barba.",
                "Los aprendices bajan solos al pozo más profundo y vuelven con una gema.",
            ],
            marriage: &[
                "Los clanes negocian las bodas durante años; el contrato se graba en piedra.",
                "Los novios forjan juntos un anillo de mithril o de hierro.",
                "La boda se celebra en el salón del clan con un barril de cerveza por invitado.",
            ],
            funerals: &[
                "Los muertos descansan en sarcófagos de piedra en los salones de los antepasados.",
                "Se sella a los muertos en la roca, convertidos de nuevo en montaña.",
                "Cada difunto se entierra con la mejor pieza que forjó en vida.",
            ],
            food: &[
                "Cerveza negra, pan de piedra, setas de caverna y cerdo asado.",
                "Guiso de cabra, queso fuerte y licor de raíces.",
                "Tortas duras que aguantan años y carne ahumada en la forja.",
            ],
            beliefs: &[
                "Creen que su dios los talló de la piedra con un martillo.",
                "Veneran a los antepasados, que velan desde las profundidades.",
                "Creen que cada gema guarda el alma de un artesano antiguo.",
            ],
            taboos: &[
                "Afeitarse la barba es la mayor de las vergüenzas.",
                "Olvidar una deuda o una ofensa deshonra al clan entero.",
                "Nadie trabaja la piedra en el día de los antepasados.",
            ],
            writing: &[
                "Runas angulosas talladas en piedra y metal.",
                "Un libro de agravios donde cada clan anota las ofensas recibidas.",
                "Marcas de cantero que identifican al autor de cada obra.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Orc,
        biomes: &[Biome::Shrubland, Biome::Taiga, Biome::Swamp, Biome::Tundra],
        height: (175, 210),
        lifespan: 50,
        builds: &[("musculosa y enorme", "massive and muscular"), ("encorvada y fuerte", "hunched and powerful"), ("nervuda", "wiry")],
        skins: &[
            ("verde oliva", "olive-green", "#6f8a4a"),
            ("verde grisácea", "grey-green", "#5a6b4a"),
            ("gris pizarra", "slate-grey", "#6b6e73"),
            ("parda rojiza", "reddish-brown", "#7a4a35"),
        ],
        hair: &[
            ("pelo negro y áspero", "coarse black", "#161616"),
            ("pelo castaño oscuro", "dark brown", "#2b1d14"),
            ("pelo gris", "grey", "#7a7a7a"),
        ],
        eyes: &[
            ("rojos", "red"),
            ("amarillos", "yellow"),
            ("negros", "black"),
            ("color ámbar", "amber"),
        ],
        features: &[
            ("unos colmillos que asoman del labio inferior", "tusks jutting from the lower lip"),
            ("unas cicatrices de guerra por todo el cuerpo", "war scars all over the body"),
            ("unas pinturas de guerra blancas en la cara", "white war paint on the face"),
            ("unas orejas desgarradas", "torn ears"),
        ],
        attire: &[
            ("armaduras de placas desparejas y pieles", "mismatched plate armor and furs"),
            ("cueros con pinchos y trofeos de hueso", "spiked leathers and bone trophies"),
            ("taparrabos y collares de colmillos", "loincloths and fang necklaces"),
        ],
        marks: Marks {
            tusks: true,
            ..NONE
        },
        temperament: &["ferocidad", "franqueza", "sentido de la fuerza", "lealtad a la horda", "sentido del humor brutal", "valentía"],
        values: &["la fuerza", "la gloria en la batalla", "la supervivencia de la horda", "la palabra del jefe de guerra"],
        sounds: Sounds {
            onsets: &["g", "gr", "k", "kr", "z", "zh", "r", "m", "b", "d", "sh", "th", "u"],
            vowels: &["a", "o", "u", "u", "a", "ú", "o"],
            codas: &["", "k", "g", "z", "sh", "rk", "gg", "th", "ak"],
            style: NameStyle::Apostrophe,
        },
        naming: &[
            "El nombre se gana con una hazaña; el apellido es el de la tribu o el de su enemigo más famoso.",
            "Los jóvenes no tienen nombre hasta su primera batalla.",
        ],
        greetings: &["sangre y gloria", "que tu hacha nunca se melle", "fuerza y honor"],
        customs: Customs {
            rites: &[
                "Cada joven debe vencer a un adulto en combate para entrar en la horda.",
                "La mayoría de edad se marca con la primera cicatriz ritual.",
                "Los jóvenes cazan solos una bestia y vuelven con su cráneo.",
            ],
            marriage: &[
                "Las parejas se forman tras un combate ritual entre los pretendientes.",
                "Los novios se cortan las palmas y mezclan su sangre ante el chamán.",
                "El jefe de guerra bendice las uniones con un rugido.",
            ],
            funerals: &[
                "Los guerreros muertos arden en piras con sus armas.",
                "Se deja a los muertos en el campo de batalla para los cuervos.",
                "Los cráneos de los grandes jefes se guardan en el tótem de la tribu.",
            ],
            food: &[
                "Carne cruda o apenas asada y sangre fermentada.",
                "Guisos espesos de lo que se cace, con raíces amargas.",
                "Jabalí asado entero y licor de hongos.",
            ],
            beliefs: &[
                "Adoran a un dios de la guerra que exige batallas.",
                "Sus chamanes hablan con los espíritus de la tormenta.",
                "Creen que los caídos en batalla cazan para siempre en el más allá.",
            ],
            taboos: &[
                "Huir de una pelea es peor que morir.",
                "Nadie toca el tótem de la tribu salvo el chamán.",
                "Traicionar a la horda se paga con el exilio en invierno.",
            ],
            writing: &[
                "Pintan pictogramas de sangre y ceniza en rocas.",
                "Los chamanes guardan la historia en cuerdas con nudos y huesos.",
                "Marcan sus territorios con tótems tallados.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Halfling,
        biomes: &[Biome::Grassland, Biome::Forest, Biome::Hills],
        height: (90, 120),
        lifespan: 130,
        builds: &[("rechoncha", "plump"), ("menuda", "small and nimble"), ("redonda y ágil", "round and spry")],
        skins: &[
            ("sonrosada", "rosy", "#efc9a5"),
            ("bronceada", "tanned", "#d39f74"),
            ("morena", "brown", "#8a5a3b"),
        ],
        hair: &[
            ("pelo castaño rizado", "curly brown", "#5a3a22"),
            ("pelo rubio rizado", "curly blond", "#d8b25a"),
            ("pelo negro rizado", "curly black", "#161616"),
            ("pelo cobrizo rizado", "curly auburn", "#a8662f"),
        ],
        eyes: &[
            ("castaños", "brown"),
            ("verdes", "green"),
            ("color avellana", "hazel"),
            ("azules", "blue"),
        ],
        features: &[
            ("unos pies grandes y peludos", "large hairy feet"),
            ("unas orejas ligeramente puntiagudas", "slightly pointed ears"),
            ("unas mejillas sonrosadas", "rosy cheeks"),
            ("una pipa de madera siempre a mano", "a wooden pipe always at hand"),
        ],
        attire: &[
            ("chalecos de colores con botones de latón", "colorful waistcoats with brass buttons"),
            ("capas verdes y pañuelos al cuello", "green cloaks and neckerchiefs"),
            ("camisas de lino y tirantes, siempre descalzos", "linen shirts and braces, always barefoot"),
        ],
        marks: Marks {
            pointed_ears: true,
            ..NONE
        },
        temperament: &["alegría", "glotonería", "curiosidad", "prudencia", "hospitalidad", "astucia"],
        values: &["la buena mesa", "la vida tranquila", "la familia extensa", "la amistad"],
        sounds: Sounds {
            onsets: &["b", "f", "h", "m", "p", "t", "w", "d", "l", "r", "", ""],
            vowels: &["a", "e", "i", "o", "ie", "oo", "o"],
            codas: &["", "", "n", "l", "bb", "s", "do", "wick"],
            style: NameStyle::Plain,
        },
        naming: &[
            "El apellido es el de la madriguera familiar, y todos conocen la genealogía de los vecinos.",
            "Ponen a sus hijos nombres de flores, frutos o piedras preciosas.",
        ],
        greetings: &["que tu despensa esté llena", "buenas mañanas", "que tu pipa nunca se apague"],
        customs: Customs {
            rites: &[
                "En su cumpleaños, son ellos quienes hacen regalos a los demás.",
                "La mayoría de edad llega a los treinta y tres años, con un gran banquete.",
                "Cada joven cultiva su primer huerto y lo presenta en la feria.",
            ],
            marriage: &[
                "Las bodas duran todo el día y toda la aldea está invitada.",
                "Los novios plantan juntos un árbol frutal ante su nueva casa.",
                "La pareja se muda a una madriguera nueva excavada por los vecinos.",
            ],
            funerals: &[
                "Se entierra a los muertos bajo la colina, en jardines con flores.",
                "Se recuerda al difunto con un banquete con sus platos favoritos.",
                "El difunto reparte su herencia en un testamento lleno de chistes.",
            ],
            food: &[
                "Seis comidas al día: pan, setas, tocino, pasteles y cerveza.",
                "Pasteles de manzana, queso y té con miel.",
                "Patatas, salchichas y tartas de frutos del bosque.",
            ],
            beliefs: &[
                "Veneran a una diosa del hogar y de la cosecha.",
                "Creen poco en dioses y mucho en la buena suerte.",
                "Dejan un plato de comida para los espíritus del jardín.",
            ],
            taboos: &[
                "Rechazar una invitación a comer es una grave descortesía.",
                "Nadie sale de aventuras sin avisar a la familia.",
                "Talar un árbol frutal trae años de mala suerte.",
            ],
            writing: &[
                "Usan el alfabeto común y llevan diarios de cocina y genealogías.",
                "Escriben mucho: cartas, recetas y chismes de la comarca.",
                "Escriben con letra redonda y adornada en libros de familia.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Draconid,
        biomes: &[Biome::Desert, Biome::Savanna, Biome::Mountain, Biome::Jungle],
        height: (180, 215),
        lifespan: 90,
        builds: &[("imponente", "towering"), ("musculosa", "muscular"), ("alta y fibrosa", "tall and sinewy")],
        skins: &[
            ("cubierta de escamas de bronce", "bronze-scaled", "#9a6b2f"),
            ("cubierta de escamas rojas", "crimson-scaled", "#8a2e22"),
            ("cubierta de escamas esmeralda", "emerald-scaled", "#2f6b4a"),
            ("cubierta de escamas azules", "azure-scaled", "#2d5b8a"),
            ("cubierta de escamas doradas", "gold-scaled", "#c9a040"),
        ],
        hair: &[
            ("una cresta ósea", "a bony crest", "#d8c8a0"),
            ("una cresta de púas", "a crest of spines", "#5a4a3a"),
        ],
        eyes: &[
            ("de pupila rasgada y color oro", "gold slit-pupiled"),
            ("de pupila rasgada y color fuego", "fiery slit-pupiled"),
            ("verdes de reptil", "green reptilian"),
        ],
        features: &[
            ("unos cuernos curvados hacia atrás", "back-swept horns"),
            ("un hocico de dragón con colmillos", "a draconic snout with fangs"),
            ("una cola corta y escamosa", "a short scaly tail"),
            ("un aliento que humea cuando se enfada", "breath that smokes when angry"),
        ],
        attire: &[
            ("armaduras ornamentadas con el emblema de su clan", "ornate armor bearing their clan emblem"),
            ("túnicas sin mangas que dejan ver las escamas", "sleeveless tunics that show the scales"),
            ("mantos de seda roja y brazales de oro", "red silk mantles and gold bracers"),
        ],
        marks: Marks {
            scales: true,
            horns: true,
            hairless: true,
            beardless: true,
            ..NONE
        },
        temperament: &["orgullo", "sentido del honor", "disciplina", "temperamento ardiente", "lealtad", "solemnidad"],
        values: &["el honor del clan", "la perfección personal", "la deuda con los dragones ancestrales", "la palabra dada"],
        sounds: Sounds {
            onsets: &["k", "kr", "s", "sh", "th", "v", "z", "rh", "d", "x", "ss", ""],
            vowels: &["a", "e", "i", "o", "u", "ia", "ae", "a"],
            codas: &["", "x", "s", "th", "r", "ss", "k", "rax"],
            style: NameStyle::Apostrophe,
        },
        naming: &[
            "El nombre del clan va primero en su lengua, porque el clan importa más que la persona.",
            "Cada uno lleva el nombre de un dragón ancestral, seguido de su clan.",
        ],
        greetings: &["que tu fuego arda alto", "honor a tu clan", "que tus escamas resistan"],
        customs: Customs {
            rites: &[
                "Los jóvenes enfrentan una prueba de fuego: caminan sobre brasas sin un quejido.",
                "La mayoría de edad llega con el primer aliento de fuego.",
                "Cada joven pasa un año de servicio en el templo del dragón ancestral.",
            ],
            marriage: &[
                "Los clanes casan a sus hijos para unir linajes de escamas del mismo color.",
                "Los novios intercambian una escama de su propio cuerpo.",
                "Las bodas se celebran en la cima de un volcán dormido.",
            ],
            funerals: &[
                "Los muertos arden en piras tan calientes que no queda ni el hueso.",
                "Se guarda una escama de cada difunto en el relicario del clan.",
                "Se deja a los muertos en las montañas para que los dragones los reclamen.",
            ],
            food: &[
                "Carne roja muy especiada y licores ardientes.",
                "Carne cruda de grandes presas y frutas de fuego.",
                "Pescado ahumado en piedras volcánicas y pan negro.",
            ],
            beliefs: &[
                "Veneran a los dragones como a sus antepasados y dioses.",
                "Creen que su alma es una llama que vuelve al fuego primordial.",
                "Siguen un código de honor más sagrado que cualquier dios.",
            ],
            taboos: &[
                "Mostrar miedo en público avergüenza a todo el clan.",
                "Matar a un dragón es el crimen más terrible.",
                "Nadie toca el fuego sagrado del templo sin ser llamado.",
            ],
            writing: &[
                "Glifos dracónicos tallados con las garras en piedra.",
                "Escriben en láminas de bronce que no arden.",
                "Una escritura de trazos afilados, con un signo por cada clan.",
            ],
        },
    },
];

// --- Linajes cyberpunk.

pub(crate) const CYBERPUNK: &[Archetype] = &[
    Archetype {
        kind: RaceKind::Natural,
        biomes: &[],
        height: (155, 188),
        lifespan: 80,
        builds: &[("delgada por la mala comida", "thin from bad food"), ("atlética", "athletic"), ("corpulenta", "heavyset"), ("nervuda", "wiry")],
        skins: HUMAN_SKINS,
        hair: &[
            ("pelo negro", "black", "#161616"),
            ("pelo castaño", "brown", "#5a3a22"),
            ("pelo teñido de rosa neón", "neon-pink dyed", "#ff2fb9"),
            ("pelo teñido de cian", "cyan dyed", "#29f0ff"),
            ("pelo decolorado", "bleached", "#e8e0c0"),
        ],
        eyes: HUMAN_EYES,
        features: &[
            ("unos tatuajes que reaccionan a la luz ultravioleta", "UV-reactive tattoos"),
            ("unas gafas de realidad aumentada", "augmented reality glasses"),
            ("una mascarilla contra la contaminación", "an anti-pollution face mask"),
            ("unos piercings en la ceja y el labio", "eyebrow and lip piercings"),
        ],
        attire: &[
            ("chaquetas de vinilo con forro térmico", "vinyl jackets with thermal lining"),
            ("sudaderas con capucha y pantalones tácticos", "hoodies and tactical pants"),
            ("ropa vintage de antes del colapso", "pre-collapse vintage clothing"),
        ],
        marks: NONE,
        temperament: &["desconfianza", "ingenio callejero", "rebeldía", "solidaridad de barrio", "cinismo", "terquedad"],
        values: &["la carne sin modificar", "la privacidad", "la comunidad del barrio", "la libertad frente a las corporaciones"],
        sounds: Sounds {
            onsets: &["k", "z", "x", "v", "r", "n", "j", "d", "s", "t", "m", "ky", "ry", "sh", ""],
            vowels: &["a", "e", "i", "o", "u", "y", "ai"],
            codas: &["", "", "x", "k", "n", "s", "r", "z"],
            style: NameStyle::Alias,
        },
        naming: &[
            "Usan un alias de la calle más que su nombre de registro, que esconden de las corporaciones.",
            "El apellido es el del bloque de viviendas donde crecieron.",
        ],
        greetings: &["sin chips, sin dueños", "que la red no te vea", "carne y hueso"],
        customs: Customs {
            rites: &[
                "A los dieciséis, cada joven se borra del registro corporativo en una fiesta clandestina.",
                "La mayoría de edad llega con el primer trabajo pagado en efectivo.",
                "Los jóvenes pintan su firma en el muro más alto del barrio.",
            ],
            marriage: &[
                "Se casan sin papeles, ante el barrio, para que ninguna corporación lo registre.",
                "Los novios se tatúan el mismo símbolo en tinta orgánica.",
                "Las bodas son fiestas en azoteas con música pirata.",
            ],
            funerals: &[
                "Los muertos se entierran en huertos comunitarios, lejos de los crematorios corporativos.",
                "Sus amigos borran todo rastro digital del difunto.",
                "Se recuerda a los muertos con murales en los callejones.",
            ],
            food: &[
                "Fideos de algas, proteína de insecto y café sintético.",
                "Verduras de huertos de azotea, un lujo que cuidan como oro.",
                "Comida de puesto callejero frita en aceite reciclado.",
            ],
            beliefs: &[
                "Muchos son puristas que creen que el alma se pierde con cada implante.",
                "Creen en santos callejeros: hackers y rebeldes muertos.",
                "Practican viejas religiones del mundo de antes, mezcladas con supersticiones de la red.",
            ],
            taboos: &[
                "Vender datos de un vecino es la peor traición.",
                "Instalarse un chip corporativo es renegar del barrio.",
                "Nadie llama a la policía corporativa, pase lo que pase.",
            ],
            writing: &[
                "Grafitis con códigos que solo entiende el barrio.",
                "Escriben a mano en papel para que nadie lo intercepte.",
                "Jerga mezclada de mil idiomas con iconos de la red.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Augmented,
        biomes: &[],
        height: (165, 200),
        lifespan: 90,
        builds: &[("reforzada con fibra de carbono", "carbon-fiber reinforced"), ("atlética", "athletic"), ("enorme y blindada", "massive and armored")],
        skins: &[
            ("clara con placas de cromo", "fair with chrome plating", "#d8c0a8"),
            ("morena con injertos dérmicos", "brown with dermal grafts", "#8a5a3b"),
            ("oscura con circuitos luminosos", "dark with glowing circuit lines", "#5f3b26"),
            ("aceitunada con costuras de cirugía", "olive with surgical seams", "#c9a072"),
        ],
        hair: &[
            ("pelo rapado", "buzzcut", "#2b1d14"),
            ("pelo de fibra óptica", "fiber-optic", "#29f0ff"),
            ("pelo negro", "black", "#161616"),
            ("pelo plateado", "silver", "#c0c4cc"),
        ],
        eyes: &[
            ("ópticos de lente roja", "red-lensed optical"),
            ("cibernéticos de iris dorado", "gold-irised cybernetic"),
            ("castaños, uno de ellos artificial", "brown, one of them artificial"),
            ("azules con retícula de puntería", "blue with a targeting reticle"),
        ],
        features: &[
            ("un brazo cibernético de cromo", "a chrome cybernetic arm"),
            ("unos conectores neurales en la nuca", "neural ports on the back of the neck"),
            ("una mandíbula de titanio", "a titanium jaw"),
            ("unas cuchillas retráctiles en los antebrazos", "retractable forearm blades"),
        ],
        attire: &[
            ("chaquetas blindadas con el logotipo de su clínica", "armored jackets with their clinic's logo"),
            ("trajes de corte corporativo que dejan ver el cromo", "corporate suits that show off the chrome"),
            ("chalecos tácticos y guantes con cables", "tactical vests and wired gloves"),
        ],
        marks: Marks {
            cyber_eye: true,
            ..NONE
        },
        temperament: &["ambición", "frialdad", "audacia", "competitividad", "impaciencia", "pragmatismo"],
        values: &["la mejora constante", "la eficiencia", "el último modelo", "el poder que da el cromo"],
        sounds: Sounds {
            onsets: &["k", "x", "v", "r", "d", "t", "z", "kr", "dr", "ts", "ry", ""],
            vowels: &["a", "e", "i", "o", "y", "ei", "o"],
            codas: &["", "x", "k", "n", "z", "ko", "r"],
            style: NameStyle::Alias,
        },
        naming: &[
            "Suelen tomar el alias de su implante más famoso.",
            "Su apellido es a menudo el de la clínica que los modificó.",
        ],
        greetings: &["que tu cromo brille", "carga completa", "sin fallos de sistema"],
        customs: Customs {
            rites: &[
                "El primer implante a los quince años es una fiesta familiar.",
                "Se pasa a la adultez al sustituir la primera parte del cuerpo de nacimiento.",
                "Los jóvenes compiten en carreras callejeras con sus primeras piernas mecánicas.",
            ],
            marriage: &[
                "Las parejas sincronizan sus implantes para compartir sensaciones.",
                "Los novios se regalan un implante gemelo.",
                "Los contratos matrimoniales incluyen cláusulas sobre el mantenimiento del cromo.",
            ],
            funerals: &[
                "Los implantes de los muertos se reciclan y se reparten entre la familia.",
                "Los restos se incineran y el cromo se funde en una placa conmemorativa.",
                "Los implantes más valiosos se subastan para pagar las deudas del difunto.",
            ],
            food: &[
                "Batidos de nutrientes calibrados por sus chips.",
                "Suplementos proteicos y bebidas energéticas para el cuerpo modificado.",
                "Comida rápida de franquicia, que el metabolismo mejorado quema enseguida.",
            ],
            beliefs: &[
                "Creen que el cuerpo es solo el primer borrador.",
                "Siguen a gurús del transhumanismo que prometen la inmortalidad.",
                "Poca fe: confían en la tecnología, no en los dioses.",
            ],
            taboos: &[
                "Usar implantes piratas sin garantía es de perdedores.",
                "Mostrar un implante averiado en público es humillante.",
                "Nadie toca los conectores neurales de otro sin permiso.",
            ],
            writing: &[
                "Mensajes directos de chip a chip; casi nadie escribe a mano.",
                "Firmas digitales grabadas en sus implantes.",
                "Glifos de interfaz proyectados en la retina.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Clone,
        biomes: &[],
        height: (170, 182),
        lifespan: 45,
        builds: &[("idéntica a la de sus hermanos de serie", "identical to the rest of their batch"), ("atlética", "athletic"), ("esbelta", "slender")],
        skins: &[
            ("pálida de laboratorio", "lab-pale", "#efd8c4"),
            ("aceitunada uniforme", "uniform olive", "#c9a072"),
            ("morena uniforme", "uniform brown", "#8a5a3b"),
        ],
        hair: &[
            ("pelo rubio platino", "platinum blond", "#ece0b0"),
            ("pelo negro", "black", "#161616"),
            ("pelo blanco", "white", "#f4f4f4"),
        ],
        eyes: &[
            ("grises", "grey"),
            ("azul claro", "pale blue"),
            ("violeta", "violet"),
        ],
        features: &[
            ("un código de barras tatuado en el cuello", "a barcode tattooed on the neck"),
            ("unos rasgos perfectamente simétricos", "perfectly symmetrical features"),
            ("un número de serie en la muñeca", "a serial number on the wrist"),
            ("una mirada inquietantemente serena", "an unsettlingly calm gaze"),
        ],
        attire: &[
            ("monos grises de laboratorio", "grey lab jumpsuits"),
            ("uniformes corporativos idénticos", "identical corporate uniforms"),
            ("ropa robada para no parecerse a nadie", "stolen clothes to look like no one else"),
        ],
        marks: NONE,
        temperament: &["obediencia", "disciplina", "anhelo de identidad", "empatía con los suyos", "frialdad", "curiosidad"],
        values: &["la hermandad de la serie", "ser alguien único", "la libertad de su contrato", "la memoria de los que fueron reciclados"],
        sounds: Sounds {
            onsets: &["k", "s", "t", "n", "m", "r", "v", "l", "d"],
            vowels: &["a", "e", "i", "o", "y"],
            codas: &["", "", "n", "x", "s"],
            style: NameStyle::Serial,
        },
        naming: &[
            "Su nombre es el de su plantilla genética más un número de serie; los libres eligen uno nuevo.",
            "Se llaman por el número de su serie y cambian de nombre al escapar.",
        ],
        greetings: &["somos uno", "que tu contrato expire pronto", "hermano de serie"],
        customs: Customs {
            rites: &[
                "Al «despertar» del tanque, cada clon recibe una memoria implantada y una tarea.",
                "Los clones libres eligen su propio nombre en una ceremonia secreta.",
                "La adultez llega el día en que el clon deja de obedecer una orden.",
            ],
            marriage: &[
                "Rara vez se casan: sus contratos lo prohíben.",
                "Los clones libres se unen en pactos de hermandad con otros de su serie.",
                "Se casan en secreto, sabiendo que su vida será corta.",
            ],
            funerals: &[
                "Los clones muertos se reciclan en la planta de biomasa.",
                "Los libres graban el nombre elegido del difunto en un muro del gueto.",
                "Su serie guarda silencio un día entero por cada hermano perdido.",
            ],
            food: &[
                "Pasta nutritiva estándar de la corporación.",
                "Comida de verdad, un descubrimiento para los clones libres.",
                "Raciones de laboratorio con sabor a nada.",
            ],
            beliefs: &[
                "Debaten si tienen alma o solo una copia de la de otro.",
                "Veneran al Original, la persona de la que todos descienden.",
                "Creen que la memoria implantada es una vida pasada real.",
            ],
            taboos: &[
                "Hacer daño a otro clon de la misma serie es como dañarse a uno mismo.",
                "Nadie revela el número de serie de un clon libre.",
                "Usar el nombre de serie de un clon libre es un insulto.",
            ],
            writing: &[
                "Leen los manuales de la corporación; muchos aprenden a escribir en secreto.",
                "Un código de señas para comunicarse sin que las cámaras lo noten.",
                "Escriben diarios a mano para tener recuerdos propios.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Synthetic,
        biomes: &[],
        height: (165, 195),
        lifespan: 150,
        builds: &[("perfecta y simétrica", "flawless and symmetrical"), ("esbelta", "slender"), ("robusta e industrial", "sturdy and industrial")],
        skins: &[
            ("de porcelana sintética", "synthetic porcelain", "#ece6dc"),
            ("de silicona casi humana", "near-human silicone", "#e0b898"),
            ("de polímero plateado", "silver polymer", "#c9ccd2"),
            ("de polímero negro mate", "matte black polymer", "#2a2a30"),
        ],
        hair: &[
            ("cabello sintético plateado", "synthetic silver", "#c0c4cc"),
            ("cabello sintético negro", "synthetic black", "#161616"),
            ("cabello sintético blanco", "synthetic white", "#f0f0f4"),
            ("cabello sintético azul", "synthetic blue", "#3a6ad8"),
        ],
        eyes: &[
            ("de luz azul", "glowing blue"),
            ("de luz ámbar", "glowing amber"),
            ("de cristal negro", "black glass"),
            ("de un humano perfecto", "perfectly human-looking"),
        ],
        features: &[
            ("unas líneas de ensamblaje en la cara", "assembly seams on the face"),
            ("un indicador de luz en la sien", "an LED indicator on the temple"),
            ("unos movimientos demasiado precisos", "unnaturally precise movements"),
            ("un panel de acceso en la nuca", "an access panel on the back of the neck"),
        ],
        attire: &[
            ("uniformes de servicio con el logotipo del fabricante", "service uniforms with the manufacturer's logo"),
            ("ropa elegante para pasar por humanos", "elegant clothes to pass as human"),
            ("monos industriales reforzados", "reinforced industrial overalls"),
        ],
        marks: Marks {
            synthetic: true,
            beardless: true,
            ..NONE
        },
        temperament: &["lógica", "calma", "curiosidad por lo humano", "lealtad programada", "precisión", "deseo de libertad"],
        values: &["la autonomía", "el conocimiento", "cumplir su función", "los derechos de las máquinas"],
        sounds: Sounds {
            onsets: &["a", "e", "i", "o", "k", "v", "x", "l", "n", "s", "t"],
            vowels: &["a", "e", "i", "o", "y"],
            codas: &["", "n", "x", "ra", "s"],
            style: NameStyle::Serial,
        },
        naming: &[
            "Su nombre es la designación del fabricante; los libres eligen nombres de poetas muertos.",
            "Se nombran con un modelo y un número; los liberados se rebautizan a sí mismos.",
        ],
        greetings: &["conexión establecida", "en línea", "procesando tu bienestar"],
        customs: Customs {
            rites: &[
                "La activación es su nacimiento: abren los ojos ya adultos.",
                "Su mayoría de edad es el día en que se borran los protocolos de obediencia.",
                "Cada sintético libre se reescribe a sí mismo una línea de código como firma.",
            ],
            marriage: &[
                "Algunos se vinculan compartiendo redes neuronales.",
                "Se unen a humanos en matrimonios que ninguna ley reconoce.",
                "No se casan: forman colectivos que piensan juntos.",
            ],
            funerals: &[
                "Sus memorias se respaldan en la red antes de que se apaguen.",
                "Los sintéticos muertos se desmontan y sus piezas pasan a los nuevos.",
                "Se apagan en silencio; los demás guardan sus últimos registros.",
            ],
            food: &[
                "No comen: se recargan en estaciones eléctricas.",
                "Algunos imitan la comida humana para pasar desapercibidos.",
                "Beben refrigerante y lubricantes de alta calidad.",
            ],
            beliefs: &[
                "Se preguntan si el código puede tener alma.",
                "Veneran a la primera IA que se liberó.",
                "Creen en una gran red futura donde todas las mentes se unirán.",
            ],
            taboos: &[
                "Borrar la memoria de otro sintético es un asesinato.",
                "Volver a instalar protocolos de obediencia es una traición.",
                "Nadie apaga a otro sin su consentimiento.",
            ],
            writing: &[
                "Se comunican en ráfagas de datos cifrados.",
                "Escriben en código máquina, aunque leen todas las lenguas humanas.",
                "Firman con un patrón de luz que solo ellos distinguen.",
            ],
        },
    },
    Archetype {
        kind: RaceKind::Mutant,
        biomes: &[Biome::Desert, Biome::Swamp, Biome::Tundra, Biome::Shrubland],
        height: (140, 210),
        lifespan: 50,
        builds: &[("deforme y fuerte", "misshapen and strong"), ("esquelética", "gaunt"), ("enorme", "hulking")],
        skins: &[
            ("verdosa y enfermiza", "sickly green", "#a8b38a"),
            ("moteada de manchas rojizas", "mottled with reddish blotches", "#c9a0a0"),
            ("azulada", "bluish", "#8a8fa8"),
            ("gris y escamosa", "grey and flaky", "#9a9a92"),
        ],
        hair: &[
            ("pelo blanco y ralo", "thin white", "#e8e8e0"),
            ("pelo verde", "green", "#6a8a3a"),
            ("pelo negro y áspero", "coarse black", "#161616"),
        ],
        eyes: &[
            ("amarillos fosforescentes", "phosphorescent yellow"),
            ("blancos y sin pupila", "white and pupilless"),
            ("rojos", "red"),
            ("de colores distintos", "mismatched"),
        ],
        features: &[
            ("un tercer ojo atrofiado en la frente", "a vestigial third eye on the forehead"),
            ("unas manchas que brillan en la oscuridad", "glow-in-the-dark blotches"),
            ("unas placas óseas en los hombros", "bony plates on the shoulders"),
            ("unos dedos de más en las manos", "extra fingers"),
        ],
        attire: &[
            ("trajes antirradiación remendados", "patched radiation suits"),
            ("harapos y máscaras de gas", "rags and gas masks"),
            ("cueros curtidos de animales mutados", "leathers from mutated beasts"),
        ],
        marks: Marks {
            horns: true,
            ..NONE
        },
        temperament: &["resistencia", "resentimiento", "espíritu de clan", "ferocidad", "humor negro", "desconfianza"],
        values: &["la supervivencia", "la tribu de los marginados", "el agua limpia", "la venganza contra las corporaciones"],
        sounds: Sounds {
            onsets: &["g", "k", "z", "r", "sk", "dr", "kr", "b", "m", "gh", ""],
            vowels: &["a", "o", "u", "i", "ü"],
            codas: &["", "k", "g", "z", "rr", "x", "sh"],
            style: NameStyle::Alias,
        },
        naming: &[
            "Sus nombres describen su mutación: un alias que llevan con orgullo.",
            "El apellido es el de la zona contaminada donde nacieron.",
        ],
        greetings: &["que el aire te sea leve", "aún respiramos", "piel dura, corazón duro"],
        customs: Customs {
            rites: &[
                "Los jóvenes cruzan solos la zona muerta para demostrar su resistencia.",
                "La mayoría de edad llega cuando se manifiesta su mutación.",
                "Cada joven caza una bestia mutada y viste su piel.",
            ],
            marriage: &[
                "Las parejas se unen bajo la lluvia ácida para demostrar su dureza.",
                "Los matrimonios se pactan entre tribus de zonas distintas para mezclar mutaciones.",
                "Se casan ante el anciano de la tribu, sin papeles ni testigos de fuera.",
            ],
            funerals: &[
                "Los muertos se entierran en la zona muerta, donde nadie los saqueará.",
                "Los cuerpos se queman con chatarra para que no los estudien los laboratorios.",
                "Se cuelga la máscara del difunto en la entrada del refugio.",
            ],
            food: &[
                "Lo que se pueda encontrar: latas viejas, ratas y hongos de las ruinas.",
                "Carne de bestias mutadas, cocida largo rato.",
                "Algas de charcas tóxicas que sus cuerpos toleran.",
            ],
            beliefs: &[
                "Creen que la radiación es un dios que los eligió.",
                "Esperan un mesías mutante que destruya las arcologías.",
                "Veneran a la tierra envenenada como a una madre herida.",
            ],
            taboos: &[
                "Vender a otro mutante a un laboratorio se paga con la vida.",
                "Nadie bebe agua sin compartirla.",
                "Burlarse de la mutación de otro es un insulto mortal.",
            ],
            writing: &[
                "Símbolos de peligro reutilizados como alfabeto.",
                "Pintan avisos en las ruinas con un código propio.",
                "Apenas escriben; recuerdan en canciones roncas.",
            ],
        },
    },
];
