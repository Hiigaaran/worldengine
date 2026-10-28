# Cambios

Todas las versiones de WorldEngine. El formato sigue
[Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [0.1.0] - 2026-10-28

Primera versión: un generador de mundos y de su historia, inspirado en la
generación de mundos y el modo Leyendas de Dwarf Fortress.

### Geografía
- Mapa de 256 × 160 casillas generado a partir de una semilla (número o texto); la misma semilla da siempre el mismo mundo.
- Placas tectónicas, cordilleras, nivel del mar configurable, temperatura por latitud y altura, y lluvia con vientos y sombra orográfica.
- Ríos con afluentes que bajan hasta el mar o los lagos, erosión y 15 biomas.
- Regiones con nombre (montañas, bosques, desiertos, mares…).

### Países, localidades y personajes
- Asentamientos fundados en los mejores lugares (ríos, costas, lagos) y países con territorio, capital y forma de gobierno.
- Personajes de interés: gobernantes, herederos, generales, eruditos…, con familia, hogar y causa de muerte.

### Historia año a año
- Crecimiento de la población: aldeas que se vuelven pueblos y ciudades, y ruinas cuando se vacían.
- Expansión de fronteras, anexiones y colonias.
- Nacimientos, muertes, sucesiones y coronaciones.
- Diplomacia y alianzas; guerras con nombre, batallas, conquistas y tratados de paz.
- Rebeliones e independencias, pestes, cambios de capital y caída de países.
- La historia se puede seguir simulando (+10 o +50 años) y es determinista.

### Interfaz
- Mapa con zoom y arrastre, en cinco modos: físico, político, elevación, temperatura y lluvia.
- Panel de detalles de cualquier casilla: región, río, país, localidad y sus personajes.
- Línea de tiempo con viaje a cualquier año, reproducción de la historia y lista de eventos filtrable.
- Modo Leyendas: una enciclopedia de países, localidades, personajes y guerras, con fichas enlazadas entre sí y con el mapa.
- Exportar la crónica del mundo en Markdown y el mundo completo en JSON.

[0.1.0]: https://github.com/Hiigaaran/worldengine/releases/tag/v0.1.0
