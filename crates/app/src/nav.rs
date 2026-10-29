//! Acciones de navegación que piden los paneles; la aplicación las aplica al
//! final de cada cuadro.

use worldengine_core::{CountryId, EventId, LocalityId, PersonId, Pos, StoryId, WarId, Year};

/// Una ficha de la enciclopedia de Leyendas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Country(CountryId),
    Locality(LocalityId),
    Person(PersonId),
    War(WarId),
    Story(StoryId),
    Event(EventId),
}

pub enum Nav {
    /// Seleccionar una casilla sin mover el mapa (clic en el mapa).
    Select(Pos),
    /// Seleccionar una casilla y centrar el mapa en ella.
    GoTo(Pos),
    /// Quitar la selección y volver a la vista general.
    Overview,
    /// Mostrar el mundo tal como era en un año, opcionalmente en un lugar.
    Travel(Year, Option<Pos>),
    /// Volver al año presente.
    Present,
    /// Abrir una ficha en el panel de Leyendas.
    Open(Page),
    /// Seleccionar un evento: abre su ficha y centra el mapa en su lugar.
    /// Con `travel`, además muestra el mundo en el año en que ocurrió.
    Event { id: EventId, travel: bool },
    /// Encuadrar el mapa sobre varios lugares (los de una historia): se centra
    /// en el más cercano al promedio de todos y se marcan en el mapa.
    Frame(Vec<Pos>),
    /// Guardar una imagen SVG con este nombre de archivo.
    SaveImage(String, String),
}
