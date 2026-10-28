# WorldEngine

Aplicación en Rust para generar un mundo ficticio y simular su historia, como base
para escribir un libro, preparar una campaña de Dungeons & Dragons o simplemente
explorar. La inspiración es la generación de mundos y el modo Leyendas de
Dwarf Fortress.

![Mapa físico](docs/img/mapa-fisico.png)

## Cómo ejecutarlo

```sh
cargo run --release -p worldengine-app      # app visual
cargo run -p worldengine-core --example summary -- Arda   # resumen en terminal
cargo test --workspace
```

En Linux, la app necesita las bibliotecas de X11/Wayland y OpenGL habituales
(por ejemplo `libxkbcommon-x11-0` y `libgl1-mesa-dri`).

En la app: la **rueda** hace zoom, **arrastrar** mueve el mapa y un **clic** muestra
los detalles de la casilla (región, río, país, localidad, personajes e historia).
Al acercarse aparecen afluentes, regiones pequeñas y nombres de ríos y aldeas.
El mapa tiene cinco capas: física, política, elevación, temperatura y lluvia. La
semilla acepta números o texto: la misma semilla produce siempre el mismo mundo.
**Exportar JSON** guarda el mundo completo en el directorio actual.

## Estructura

| Crate | Contenido |
| --- | --- |
| `crates/core` (`worldengine-core`) | Modelo de datos, generación por etapas, consultas y serialización. Sin dependencias gráficas. |
| `crates/app` (`worldengine-app`) | Interfaz con `egui`: mapa físico y político, panel de detalles y línea de tiempo. |

El modelo se organiza por capas de detalle:

```
Mundo → Mapa (casillas: elevación, temperatura, lluvia, bioma, región, dueño)
      → Regiones → Países → Localidades → Personajes → Eventos
```

Todas las entidades se referencian por ids tipados (`RegionId`, `CountryId`, ...)
que son índices en las colecciones del `World`. Los eventos son la columna
vertebral de la historia: la línea de tiempo y el futuro modo Leyendas se
construyen a partir de ellos.

La generación corre por etapas, y cada una usa su propio flujo aleatorio
derivado de la semilla, para que cambiar una etapa no altere a las demás.

**Geografía** (`crates/core/src/geography.rs`):

1. *Relieve*: placas tectónicas continentales u oceánicas que se mueven; donde
   chocan se levantan cordilleras y donde se separan se abren fosas. El ruido
   fractal añade colinas y detalle.
2. *Clima*: la temperatura depende de la latitud y la altura. La lluvia la traen
   vientos dominantes según la latitud (alisios, vientos del oeste, polares): se
   cargan de humedad sobre el mar y la descargan al subir montañas, dejando
   desiertos en la sombra de lluvia.
3. *Agua*: inundación por prioridad desde el mar. Las depresiones hondas quedan
   como lagos, cada casilla drena hacia una vecina y el caudal acumula la lluvia
   de toda la cuenca. Donde el caudal es grande nace un río; los brazos menores
   son afluentes. Los ríos excavan sus valles.
4. *Biomas* (15) según elevación, temperatura y lluvia, y *regiones* con nombre.

**Civilizaciones** (`crates/core/src/generate.rs`): las capitales y pueblos
buscan tierra fértil junto a ríos, lagos y costas; los países se expanden desde
su capital y las montañas, desiertos y ríos frenan la expansión, así que las
fronteras siguen el relieve.

## Plan

1. **Base y modelo de datos.** Workspace, modelo, semilla reproducible, mundo de
   prueba y app con mapa y detalles. *(hecho)*
2. **Geografía y mapa.** Placas tectónicas, clima con vientos, ríos y lagos,
   15 biomas, regiones con nombre y mapa con relieve, capas y zoom con más
   detalle. *(hecho)*
3. **Civilizaciones e historia.** Simulación año a año: crecimiento, fundaciones,
   guerras, alianzas, sucesiones y muertes; territorios que cambian.
4. **Modo Leyendas.** Línea de tiempo deslizable que muestra el mapa en cada año,
   búsqueda de personajes, sitios y eventos, y exportación a crónica en texto.
