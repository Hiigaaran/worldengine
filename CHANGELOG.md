# Cambios

Todas las versiones de WorldEngine. El formato sigue
[Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [Sin publicar]

### Enciclopedia e intrigas
- Cada evento recuerda el hecho que lo provocó: la batalla que costó una ciudad, la muerte que dejó vacío un trono, el agravio que desató una guerra.
- Nuevos hechos de personajes: bodas (las de casas extranjeras acercan a los países y pueden sellar alianzas; el consorte viudo puede heredar el trono), asesinatos y conjuras descubiertas por ambición, guerra, rivalidad o venganza, usurpaciones, ejecuciones y deudas de sangre que los hijos intentan saldar.
- Las guerras estallan por motivos concretos (un asesino descubierto, una derrota que vengar, una secesión, una alianza rota) y reciben nombres según su causa o el tiempo de aquel año: la Guerra de la Lluvia, la Guerra del Puñal, la Segunda Guerra de los Robles…
- Historias: los eventos se agrupan en relatos con título y texto de contexto. Guerras, rivalidades entre dos países, intrigas, bodas, pestes, rebeliones, dinastías y épocas de paz, enlazadas por causas y consecuencias.
- En Leyendas, nueva categoría «Historias» (con filtro por tipo) y ficha de cada historia; las fichas de países, localidades, personajes y guerras muestran sus historias.
- La crónica exportada incluye una sección «Historias».

### Clima y batallas en el mapa
- Clima de cada año: unas decenas de frentes de lluvia y tormentas cruzan el mapa con los vientos dominantes y se promedia cuánto llovió en cada casilla respecto a lo normal.
- Nubes animadas con lluvia donde más llovió ese año, nubes oscuras con rayos donde hubo tormentas; se vuelven transparentes al pasar el mouse encima.
- El modo de mapa «Lluvia» muestra la lluvia del año elegido.
- Explosiones animadas donde hubo batallas o conquistas en el año mostrado.
- Resumen del clima del año en el mapa (región más lluviosa, más seca, tormentas y combates) y lluvia del año al pasar el mouse por una casilla.
- Botones «Clima» y «Batallas» para mostrar u ocultar las animaciones.

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
