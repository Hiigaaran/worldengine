//! Identificadores tipados para cada entidad del mundo.
//!
//! Cada id es el índice de la entidad dentro de su colección en [`crate::World`],
//! así que buscar una entidad por id es acceso directo a un `Vec`.

use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($($(#[$doc:meta])* $name:ident),* $(,)?) => {
        $(
            $(#[$doc])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
            #[serde(transparent)]
            pub struct $name(pub u32);

            impl $name {
                pub fn index(self) -> usize {
                    self.0 as usize
                }
            }
        )*
    };
}

define_id!(
    /// Región geográfica (zona continua con un bioma dominante).
    RegionId,
    /// Río o afluente.
    RiverId,
    /// País o entidad política.
    CountryId,
    /// Localidad: aldea, pueblo o ciudad.
    LocalityId,
    /// Personaje de interés.
    PersonId,
    /// Evento histórico.
    EventId,
    /// Guerra entre dos países.
    WarId,
    /// Historia: un grupo de eventos enlazados por causa y consecuencia.
    StoryId,
);
