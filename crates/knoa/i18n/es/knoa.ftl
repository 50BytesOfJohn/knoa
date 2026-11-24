## Terms
## One word per concept, used everywhere in this language.

-knoa = Knoa
-workspace = espacio
-bookmark = marcador
-folder = carpeta
-tag = etiqueta
-summary = resumen
-organize = organizar

## Menus

menu-about = Acerca de { $name }
menu-check-for-updates = Buscar actualizaciones…
menu-settings = Ajustes…
menu-services = Servicios
menu-hide = Ocultar { $name }
menu-hide-others = Ocultar otros
menu-show-all = Mostrar todo
menu-quit = Salir de { $name }
menu-help = Ayuda
menu-whats-new = Novedades de { $name } { $version }
menu-report-issue = Reportar un problema…

## Onboarding

onboarding-skip-setup = Omitir configuración →
onboarding-back = Atrás
onboarding-next = Siguiente
onboarding-hello-title = Hola, soy { -knoa }.
onboarding-hello-lead = Guardas cosas sin parar como una ardilla preparándose para el invierno. Yo las vuelvo a encontrar. Leo los marcadores de tu navegador, averiguo qué es cada uno y lo mantengo todo en orden.
onboarding-hello-questions = Tres preguntas rápidas. O ninguna, si tienes prisa.
onboarding-hello-start = Empecemos
onboarding-hello-defaults = Usar ajustes predeterminados
onboarding-hello-have-account = ¿Ya tienes una cuenta de { -knoa }?
onboarding-hello-sign-in = Iniciar sesión
onboarding-browser-title = ¿Dónde viven tus marcadores?
onboarding-browser-lead = Sigue guardando marcadores como siempre. Notaré los nuevos en un par de segundos. Solo leo; nunca toco los marcadores de tu navegador.
onboarding-browser-count =
    { $count ->
        [one] { $count } marcador
       *[other] { $count } marcadores
    }
onboarding-browser-none-found = No encontré ningún navegador basado en Chromium aquí (Helium, Chrome, Brave, Edge, Vivaldi). No pasa nada: puedes elegir uno más tarde en Ajustes.
onboarding-browser-bring-in = Traer { $count }
onboarding-ai-title = ¿Quieres que me encargue de ordenar?
onboarding-ai-lead = Con una clave de OpenRouter leeré cada página nueva y le pondré un tipo, un resumen, etiquetas y una carpeta. Es tu clave y tu cuenta: unos 17 centavos de dólar por cada mil marcadores.
onboarding-ai-key-placeholder = Pega tu clave: sk-or-v1-…
onboarding-ai-check-key = Comprobar clave
onboarding-ai-checking = Llamando a la puerta de OpenRouter…
onboarding-ai-key-works = Esa clave funciona.
onboarding-ai-key-missing = Pega una clave primero, u omite este paso.
onboarding-ai-key-unknown = OpenRouter no reconoce esa clave. ¿La revisas?
onboarding-ai-key-check-failed = No se pudo comprobar la clave: { $error }
onboarding-ai-get-key = Obtener una clave ↗
onboarding-ai-without-key = Sin ella, también obtengo imágenes y descripciones.
onboarding-ai-skip = Omitir, sin IA
onboarding-library-title = Por último: ¿cómo se llama esta biblioteca?
onboarding-library-default-name = Personal
onboarding-library-house-rules = ¿Alguna regla de la casa para organizar?
onboarding-library-instructions-placeholder = Opcional. Escríbelo como se lo dirías a un amigo.
onboarding-library-example-apart = Mantén el trabajo y lo personal por separado.
onboarding-library-example-recipes = Etiqueta las recetas por tipo de cocina.
onboarding-library-example-rust = Todo lo relacionado con Rust va en una carpeta Rust.
onboarding-library-example-short = Resúmenes cortos, sin relleno.
onboarding-library-add-example = + { $example }
onboarding-library-finish = Finalizar
onboarding-done-title = Todo listo.
onboarding-done-no-browser = Todavía no hay ningún navegador, así que tu biblioteca empieza vacía. Elige uno en Ajustes cuando quieras.
onboarding-done-bringing-in = Trayendo tus marcadores…
onboarding-done-count =
    { $count ->
        [one] { $count } marcador listo, ya puedes buscarlo.
       *[other] { $count } marcadores listos, ya puedes buscarlos.
    }
onboarding-done-organize-existing =
    { $count ->
        [one] Los marcadores nuevos se organizan solos. ¿Quieres que también revise el marcador que ya tienes? Unos { $cost }.
       *[other] Los marcadores nuevos se organizan solos. ¿Quieres que también revise los { $count } que ya tienes? Unos { $cost }.
    }
onboarding-done-organize = Sí, organízalos
onboarding-done-open-library = Abrir mi biblioteca

## Settings

settings-title = Ajustes
settings-workspaces = Espacios
settings-new-workspace = Nuevo espacio
settings-nav-general = General
settings-nav-appearance = Apariencia
settings-nav-ai = IA
settings-nav-account = Cuenta
settings-nav-organize = Organizar
settings-nav-mcp = MCP
settings-general-title = General
settings-general-about = Dónde guarda { -knoa } tu biblioteca, cómo empezó y cómo se actualiza.
settings-general-library = Biblioteca
settings-general-location = Ubicación de la biblioteca
settings-general-show-in-finder = Mostrar en el Finder
settings-general-setup = Configuración inicial
settings-general-run-setup-again = Volver a configurar
settings-general-run-setup-about = Elige navegadores, IA e instrucciones desde el inicio. Tus marcadores se conservan.
settings-general-run-setup = Iniciar configuración
settings-general-language = Idioma
settings-general-language-label = Idioma de la app y la IA
settings-general-language-about = Menús, ajustes y fechas, y los resúmenes y etiquetas que escribe la IA.
settings-general-language-note = Los resúmenes y etiquetas que ya existen se quedan en su idioma anterior. Volver a organizar tu biblioteca los reescribe.
settings-general-language-organize = Volver a organizar…
settings-general-language-organize-confirm = Organizar todos
settings-general-language-waiting = Podrás volver a organizar cuando tu cuenta tenga el nuevo idioma.
settings-updates = Actualizaciones
settings-updates-version = { -knoa } { $version }
settings-updates-self-built = Esta compilación no se actualiza sola. Las versiones publicadas en GitHub sí.
settings-updates-checked-today = Última comprobación hoy a las { $time }
settings-updates-checked-yesterday = Última comprobación ayer a las { $time }
settings-updates-checked-on = Última comprobación el { $date }
settings-updates-not-checked = Sin comprobar aún
settings-updates-check-now = Buscar ahora
settings-updates-check-automatically = Buscar actualizaciones automáticamente
settings-updates-check-automatically-about = Una vez al día, en segundo plano.
settings-updates-install-automatically = Instalar actualizaciones automáticamente
settings-updates-install-automatically-about = Descarga nuevas versiones y las instala al salir de { -knoa }.
settings-appearance-title = Apariencia
settings-appearance-about = Cómo se ve { -knoa }. Los cambios se ven al instante.
settings-appearance-theme = Tema
settings-appearance-light = Claro
settings-appearance-dark = Oscuro
settings-appearance-library = Biblioteca
settings-appearance-summaries = Mostrar resúmenes en las tarjetas
settings-appearance-summaries-about = Hasta tres líneas debajo de cada título.
settings-appearance-up-next = Mostrar la franja de A continuación
settings-appearance-up-next-about = La fila con lo que quieres leer o ver, en la parte superior de Todo.
settings-appearance-events = Mostrar eventos como
settings-appearance-events-about = Un calendario mensual con lo próximo, o una sola lista. Solo en este Mac.
settings-appearance-library-order = Las tarjetas se agrupan por fecha. Al buscar se ordenan por relevancia.
settings-reorganize-title = Volver a organizar
settings-reorganize-folders-confirm = Clasificar todos
settings-reorganize-everything-confirm = Organizar todos
settings-organize-title = Organizar
settings-organize-about = Haz que la IA organice los marcadores a los que aún no ha llegado, o que repase los que ya organizó.
settings-organize-scope = Marcadores en
settings-organize-scope-all = Todos los espacios
settings-organize-new = Sin organizar
settings-organize-counting = Contando marcadores…
settings-organize-none = Aquí todo está organizado.
settings-organize-waiting =
    { $count ->
        [one] { $count } marcador aún no está organizado
       *[other] { $count } marcadores aún no están organizados
    }
settings-organize-ai-off = Organizar requiere IA
settings-organize-ai-off-about = Conecta tu propia clave u obtén Pro en la página de IA.
settings-organize-open-ai = Configurar IA
settings-reorganize-folders-label = Solo carpetas
settings-reorganize-folders-about = Clasifica los marcadores organizados en las carpetas que tienes ahora. No vuelve a leer las páginas, así que es rápido y barato.
settings-reorganize-folders-start = Clasificar de nuevo…
settings-reorganize-everything-label = Todo
settings-reorganize-everything-about = Vuelve a leer cada página y rehace su resumen, campos, etiquetas y carpetas con las instrucciones y el idioma actuales.
settings-reorganize-everything-start = Volver a organizar…
settings-reorganize-kept = Las carpetas que pusiste o quitaste tú se quedan como están.
settings-reorganize-queued =
    { $count ->
        [one] Se puso { $count } marcador en cola.
       *[other] Se pusieron { $count } marcadores en cola.
    }
settings-account-synced-today = Última sincronización hoy a las { $time }
settings-account-synced-yesterday = Última sincronización ayer a las { $time }
settings-account-synced-on = Última sincronización el { $date }
settings-account-not-synced = Sin sincronizar aún
settings-account-start-trial = Empezar prueba gratuita
settings-account-upgrade = Pasar a Pro
settings-account-manage-subscription = Administrar suscripción
settings-account-usage = Uso
settings-account-usage-device = { $used } de { $limit } marcadores
settings-account-usage-cloud = { $used } de { $limit } marcadores, { $organized } de { $allowance } organizados este mes
settings-account-plan-free = Plan gratuito.
settings-account-plan-trial = Prueba de Pro.
settings-account-plan-pro = Plan Pro.
settings-account-plan-ends = Termina el { $date }.
settings-account-plan-trial-ends = La prueba termina el { $date }.
settings-account-plan-renews = Se renueva el { $date }.
settings-account-organized-on-device = La organización usa tu propia clave de IA, en este Mac.
settings-account-organized-in-cloud = { -knoa } organiza tus marcadores en la nube.
settings-account-title = Cuenta
settings-account-about = Sincroniza tu biblioteca con una cuenta de { -knoa }.
settings-account-unavailable = La sincronización en la nube no está disponible en esta compilación.
settings-account-couldnt-start = No pudo iniciarse: { $reason }
settings-account-cloud-mode = Modo nube
settings-account-what-it-is = El modo nube sincroniza tu biblioteca entre tus dispositivos y con { -knoa } en la web. Con Pro, { -knoa } también organiza tus marcadores en la nube, sin necesidad de tu propia clave.
settings-account-privacy = En el modo nube, tu biblioteca se guarda en los servidores de { -knoa }, que leen tus páginas para organizarlas. No tiene cifrado de extremo a extremo. Si cierras sesión, todo se queda en este Mac.
settings-account-finish-signing-in = Termina de iniciar sesión en tu navegador…
settings-account-finish-signing-in-about = { -knoa } abrió la página de inicio de sesión en tu navegador.
settings-account-cancel = Cancelar
settings-account-knoa-account = Cuenta de { -knoa }
settings-account-sign-in-about = Inicia sesión o crea una cuenta en tu navegador.
settings-account-sign-in = Iniciar sesión
settings-account-other-account = Esta biblioteca está sincronizada con otra cuenta de { -knoa }. No se sincronizará nada hasta que elijas.
settings-account-sync-with = Sincronizar con { $email } en su lugar
settings-account-sync-with-this-account = Sincronizar con esta cuenta en su lugar
settings-account-sync-with-about = La biblioteca se combina con esta cuenta. La otra cuenta conserva su copia.
settings-account-use-this-account = Usar esta cuenta
settings-account-this-account = Esta cuenta
settings-account-asking = Sesión iniciada. Consultando la cuenta en el servidor…
settings-account-sync = Sincronización
settings-account-connecting = Conectando…
settings-account-up-to-date = Al día
settings-account-syncing =
    { $count ->
        [one] Sincronizando { $count } cambio…
       *[other] Sincronizando { $count } cambios…
    }
settings-account-offline = Sin conexión. Los cambios se sincronizarán cuando { -knoa } pueda conectar con el servidor.
settings-account-sync-stopped = Sincronización detenida: { $error }
settings-account-notice-plan-limit =
    { $unsent ->
        [one] Tu plan admite { $limit } marcadores, así que { $unsent } marcador nuevo quedó solo en este Mac.
       *[other] Tu plan admite { $limit } marcadores, así que { $unsent } marcadores nuevos quedaron solo en este Mac.
    }
settings-account-notice-rejected = Algunos cambios no se pudieron sincronizar y quedaron solo en este Mac.
settings-account-notice-too-large = Algunos cambios eran demasiado grandes para sincronizar y quedaron solo en este Mac.
settings-account-notice-allowance-used-up = Se agotó el cupo de organización de este mes. { -knoa } guarda tus solicitudes de organización y las reanuda automáticamente cuando haya cupo disponible.
settings-account-sign-out = Cerrar sesión
settings-account-sign-out-about = Tu biblioteca se queda en este Mac. Vuelve a iniciar sesión para seguir sincronizando.
settings-ai-access-about = Deja que apps de IA como Claude y ChatGPT lean y administren tus marcadores.
settings-ai-access-copy = Copiar
settings-ai-access-url = URL del conector
settings-ai-access-default = Acceso predeterminado
settings-ai-access-same-as-default = Igual que el predeterminado
settings-ai-access-off = Desactivado
settings-ai-access-read = Solo lectura
settings-ai-access-full = Acceso completo
settings-ai-access-not-linked = Disponible en cuanto este espacio se vincule a la nube.
settings-ai-access-saving = Guardando…
settings-ai-access-loading = Cargando acceso para IA…
settings-ai-access-save-again = Volver a guardar el acceso para IA
settings-ai-access-load-again = Volver a cargar el acceso para IA
settings-ai-access-retry = Reintentar
settings-ai-access-needs-pro = El acceso para IA requiere { -knoa } Pro
settings-mcp-title = MCP
settings-mcp-sign-in = Inicia sesión para conectar apps de IA
settings-mcp-sign-in-about = Las apps de IA llegan a tu biblioteca a través de tu cuenta de { -knoa }.
settings-mcp-open-account = Abrir Cuenta
settings-mcp-pro-about = Con Pro, apps de IA como Claude y ChatGPT pueden buscar, leer y organizar tu biblioteca.
settings-mcp-copied = Copiado
settings-mcp-connect = Conectar
settings-mcp-how = Cómo conectar
settings-mcp-how-copy = Copia la URL del conector.
settings-mcp-how-add = En Claude, ChatGPT u otra app de IA, agrégala como conector personalizado.
settings-mcp-how-sign-in = Inicia sesión en { -knoa } cuando la app te lo pida.
settings-mcp-how-about = Una app de IA conectada puede leer tus marcadores, incluido el texto guardado de sus páginas, en cada espacio al que tenga acceso. Los cambios de aquí se aplican a partir de su próxima solicitud.
settings-mcp-access = Acceso
settings-mcp-default-off = Las apps de IA no pueden acceder a los espacios que usan el valor predeterminado.
settings-mcp-default-read = Las apps de IA pueden buscar y leer marcadores en los espacios que usan el valor predeterminado.
settings-mcp-default-full = Las apps de IA también pueden agregar, editar y mover marcadores a la Papelera, y administrar carpetas y espacios.
settings-ai-title = IA
settings-ai-about = { -knoa } también funciona sin IA. Con ella, los nuevos marcadores obtienen un tipo, un resumen, etiquetas y carpetas.
settings-ai-provider = Proveedor
settings-ai-cloud-note = Con Pro, { -knoa } organiza tus marcadores en la nube. La clave que conectaste se queda en este Mac para cuando no tengas sesión iniciada.
settings-ai-key = Clave de OpenRouter
settings-ai-key-about = Tu propia clave; las solicitudes van desde este Mac directo a OpenRouter.
settings-ai-connect = Conectar
settings-ai-checking = Comprobando la clave…
settings-ai-key-connected = Conectada. Los marcadores nuevos se organizarán a partir de ahora.
settings-ai-key-failed = No funcionó: { $error }.
settings-ai-no-key = ¿Todavía no tienes clave?
settings-ai-no-key-about = Con unos pocos dólares de crédito organizas miles de marcadores.
settings-ai-get-key = Obtener una clave
settings-ai-connected = Conectado
settings-ai-disconnect = Desconectar
settings-ai-openrouter-says = OpenRouter dice: { $problem }. Revisa tu clave o tu saldo.
settings-ai-models = Modelos
settings-ai-included-in-pro = Incluido en Pro, dentro de su cupo mensual.
settings-ai-cost = Unos { $cost } para organizarlos todos.
settings-ai-organize = Organizar
settings-ai-pro-title = Deja que { -knoa } lo haga con Pro
settings-ai-pro-pitch = { -knoa } organiza tus marcadores en la nube. Sin clave que configurar, sin cobros por solicitud y con tu biblioteca sincronizada en todos tus dispositivos.
settings-ai-pro-get = Obtener Pro
settings-ai-pro-upgrade = Pasar a Pro
settings-ai-pro-have = Ya tengo Pro
settings-ai-usage = Uso
settings-ai-usage-none = Aún no hay solicitudes de IA.
settings-ai-usage-recent =
    { $days ->
        [one] Último día
       *[other] Últimos { $days } días
    }
settings-ai-usage-all-time = Todo el historial
settings-ai-usage-by-model =
    { $days ->
        [one] Por modelo, último día
       *[other] Por modelo, últimos { $days } días
    }
settings-ai-usage-records = Registros de { $provider }
settings-ai-usage-records-about = Cada solicitud, con el cargo exacto.
settings-ai-usage-open = Abrir
settings-ai-usage-requests =
    { $count ->
        [one] { $count } solicitud
       *[other] { $count } solicitudes
    }
settings-ai-usage-tokens = { $input } tokens de entrada, { $output } de salida
settings-ai-usage-unmetered =
    { $count ->
        [one] { $count } sin conteo de tokens
       *[other] { $count } sin conteo de tokens
    }
settings-ai-usage-unpriced =
    { $count ->
        [one] { $count } sin precio
       *[other] { $count } sin precio
    }
settings-workspace-new = Nuevo espacio
settings-workspace-new-about = Una biblioteca aparte, como Trabajo o Personal, con sus propios navegadores y carpetas.
settings-workspace-shown-now = Visible ahora
settings-workspace-show-it = Mostrar
settings-workspace-name-and-color = Nombre y color
settings-workspace-name = Nombre
settings-workspace-name-too-long = El nombre puede tener como máximo { $max } caracteres.
settings-workspace-color = Color
settings-workspace-no-browser = No se encontró ningún navegador basado en Chromium. { -knoa } lee Helium, Chrome, Brave, Edge, Vivaldi y Chromium.
settings-workspace-browsers-note = Sin ninguno activado, los enlaces solo se agregan a mano (⌘N). Un mismo navegador puede alimentar varios espacios.
settings-workspace-open-in = Abrir marcadores en
settings-workspace-browsers = Navegadores
settings-workspace-open-in-about = Dónde se abren los enlaces de este espacio.
settings-workspace-organize-again-about = Después de cambiar las instrucciones o etiquetas, clasifica u organiza de nuevo los marcadores de este espacio.
settings-workspace-open-organize = Organizar…
settings-workspace-default-browser = Navegador predeterminado
settings-workspace-by-hand = Solo enlaces agregados a mano.
settings-workspace-synced-from-one = Sincronizado desde { $browser }.
settings-workspace-synced-from-many = Sincronizado desde { $browsers } y { $last }.
settings-workspace-organizing = Organización
settings-workspace-instructions = Instrucciones
settings-workspace-instructions-about = Cómo debe la IA clasificar, etiquetar y resumir lo que llega aquí.
settings-workspace-instructions-too-long = Las instrucciones pueden tener como máximo { $max } caracteres.
settings-workspace-screening = Conservar solo los marcadores que encajen
settings-workspace-screening-about = Desactivado: todo lo del navegador llega aquí. Activado: la IA aparta lo que no encaja, y puedes conservarlo de todas formas.
settings-workspace-guideline = Qué va aquí
settings-workspace-guideline-too-long = Esta guía puede tener como máximo { $max } caracteres.
settings-workspace-name-placeholder = Trabajo, Personal…
settings-workspace-instructions-placeholder = p. ej., Etiqueta las recetas por tipo de cocina. Resúmenes cortos, sin relleno.
settings-workspace-guideline-placeholder = p. ej., Solo cosas de trabajo: ingeniería, diseño y nuestros clientes.
settings-workspace-create = Crear espacio
settings-workspace-delete = Eliminar
settings-workspace-delete-label = Eliminar este espacio
settings-workspace-delete-about = Sus marcadores y carpetas se eliminarán con él. Tus navegadores conservan sus marcadores.
settings-workspace-delete-button = Eliminar…
settings-workspace-delete-confirm = Eliminar para siempre
settings-tags-title = Lista de etiquetas
settings-tags-label = Etiquetas que debe usar la IA
settings-tags-about = Cuando una de estas encaja con un marcador, la IA la usa en lugar de una propia. Una pista corta le indica cuándo encaja. También puedes hacer clic derecho en una etiqueta de un marcador para agregarla a la lista.
settings-tags-add-placeholder = Agregar una etiqueta
settings-tags-hint-placeholder = Cuándo usarla (opcional)
settings-tags-remove = Quitar de la lista
settings-tags-only-listed = Usar solo etiquetas de esta lista
settings-tags-only-listed-about = Desactivado: la IA agrega sus propias etiquetas para lo que la lista no cubre. Activado: los marcadores solo reciben etiquetas de la lista, o ninguna si nada encaja.
settings-tags-only-listed-about-empty = Agrega una etiqueta primero. Desactivado: la IA agrega sus propias etiquetas para lo que la lista no cubre. Activado: los marcadores solo reciben etiquetas de la lista, o ninguna si nada encaja.
settings-tags-note = Los marcadores que se organicen a partir de ahora recibirán estas etiquetas. Para aplicarlas a los ya organizados, vuelve a organizar.
settings-tags-count = { $count } de { $max }

## Sidebar

sidebar-status-ai-needs-attention = La IA necesita atención
sidebar-status-fix = Resolver
sidebar-sync-stopped = Sincronización detenida
sidebar-retry = Reintentar
sidebar-sync-paused = Sincronización en pausa
sidebar-status-changes-not-synced = Los cambios no se sincronizaron
sidebar-status-view = Ver
sidebar-status-plan-limit = Límite del plan alcanzado
sidebar-status-upgrade = Mejorar
sidebar-status-organizing = Organizando { $current } de { $total }…
sidebar-status-ai-off = IA desactivada
sidebar-status-turn-on = Activar
sidebar-status-offline = Sin conexión. Los cambios se sincronizarán luego.
sidebar-syncing = Sincronizando…
sidebar-sync-offline =
    { $count ->
        [0] Sin conexión
        [one] Sin conexión · { $count } cambio pendiente
       *[other] Sin conexión · { $count } cambios pendientes
    }
sidebar-synced-just-now = Sincronizado ahora mismo
sidebar-synced-minutes-ago =
    { $minutes ->
        [one] Sincronizado hace { $minutes } min
       *[other] Sincronizado hace { $minutes } min
    }
sidebar-synced-hours-ago =
    { $hours ->
        [one] Sincronizado hace { $hours } h
       *[other] Sincronizado hace { $hours } h
    }
sidebar-account-settings = Ajustes de cuenta…
sidebar-open-web = Abrir { -knoa } en la web
sidebar-sign-out = Cerrar sesión
sidebar-sign-in = Iniciar sesión…
sidebar-finish-in-browser = Termina en el navegador
sidebar-cancel = Cancelar
sidebar-plan-pro = { -knoa } Pro
sidebar-plan-free = Plan gratuito
sidebar-signed-in = Sesión iniciada
sidebar-manage-plan = Administrar plan…
sidebar-upgrade-to-pro = Pasar a Pro
sidebar-this-mac = Este Mac
sidebar-knoa-account = Cuenta de { -knoa }
sidebar-place-everything = Todo
sidebar-place-gallery = Galería
sidebar-place-events = Eventos
sidebar-place-up-next = A continuación
sidebar-place-unsorted = Sin carpeta
sidebar-place-not-organized = Sin organizar
sidebar-place-set-aside = Apartado
sidebar-place-trash = Papelera
sidebar-settings = Ajustes
sidebar-folders = Carpetas
sidebar-new-folder = Nueva carpeta
sidebar-folder-new-subfolder = Nueva subcarpeta
sidebar-folder-edit = Editar carpeta…
sidebar-folder-delete = Eliminar…
sidebar-folder-move-to = Mover a
sidebar-folder-top-level = Nivel superior
sidebar-add-link = Agregar enlace
sidebar-workspaces-title = Espacios
sidebar-workspaces-new = Nuevo espacio
sidebar-workspaces-by-hand = Enlaces agregados a mano
sidebar-workspaces-from = Desde { $browsers }

## Main

main-blocked-browsers =
    { $count ->
        [one] { $browsers } está instalado, pero macOS mantiene privados sus marcadores. Dale a { -knoa } Acceso total al disco y luego reinicia { -knoa }.
       *[other] { $browsers } están instalados, pero macOS mantiene privados sus marcadores. Dale a { -knoa } Acceso total al disco y luego reinicia { -knoa }.
    }
main-open-privacy-settings = Abrir ajustes de Privacidad
main-cancel = Cancelar
main-delete = Eliminar
main-save = Guardar
main-folder-new-title = Nueva carpeta
main-folder-edit-title = Editar carpeta
main-folder-name-placeholder = Nombre de la carpeta
main-folder-description-placeholder = ¿Qué va aquí? La IA usa esto para clasificar marcadores.
main-folder-name-too-long = El nombre puede tener como máximo { $max } caracteres.
main-folder-description-too-long = La descripción puede tener como máximo { $max } caracteres.
main-folder-auto-assign = Dejar que la IA agregue marcadores
main-folder-auto-assign-about = Los nuevos marcadores que encajen irán aquí automáticamente.
main-glyph-read = Lectura
main-glyph-article = Artículos
main-glyph-code = Código
main-glyph-write = Escritura
main-glyph-work = Trabajo
main-glyph-tag = Ofertas
main-glyph-bag = Compras
main-glyph-plane = Viajes
main-glyph-cup = Comida
main-glyph-music = Música
main-glyph-play = Video
main-glyph-photo = Fotos
main-glyph-bulb = Ideas
main-glyph-heart = Salud
main-glyph-home = Hogar
main-glyph-star = Favoritos
main-glyph-clock = Más tarde
main-glyph-flask = Ciencia
main-glyph-globe = Web
main-glyph-person = Personas
main-glyph-box = Archivo
main-glyph-inbox = Bandeja de entrada
main-glyph-leaf = Naturaleza
main-glyph-bolt = Rápido
main-glyph-lock = Privado
main-glyph-pin = Lugares
main-glyph-chat = Hilos
main-glyph-game = Juegos
main-folder-icon-none = Carpeta
main-folder-icon-emoji = Emoji
main-folder-parent = Carpeta superior
main-folder-parent-none = Ninguna
main-folder-emoji-placeholder = Escribe o pega
main-folder-any-emoji = Cualquier emoji
main-folder-custom-color = Personalizado
main-delete-folder-title = ¿Eliminar “{ $name }”?
main-delete-folder-detail =
    { $count ->
        [0] Sus marcadores se conservan en tu biblioteca.
        [one] Su subcarpeta también se eliminará. Los marcadores se conservan en tu biblioteca.
       *[other] Sus { $count } subcarpetas también se eliminarán. Los marcadores se conservan en tu biblioteca.
    }
main-open = Abrir
main-open-in-background = Abrir en segundo plano
main-copy-link = Copiar enlace
main-copied = Copiado
main-show-details = Mostrar detalles
main-view-picture = Ver imagen
main-copy-image = Copiar imagen
main-save-image = Guardar imagen…
main-add-to-up-next = Agregar a A continuación
main-remove-from-up-next = Quitar de A continuación
main-add-to-folder = Agregar a carpeta
main-remove-from-folder = Quitar de { $folder }
main-refresh-thumbnail = Actualizar miniatura
main-refreshing-thumbnail = Actualizando miniatura…
main-organize = Organizar
main-organize-again = Volver a organizar
main-organizing = Organizando…
main-move-to-trash = Mover a la Papelera
main-restore = Restaurar
main-delete-now = Eliminar ahora
main-open-link = Abrir enlace
main-card-failed = No se pudo organizar
main-card-not-organized = Aún sin organizar
main-empty-no-events-match = Ningún evento coincide
main-empty-no-events-match-body = Prueba con menos palabras o un filtro de fecha como starts>2026-12-01.
main-empty-nothing-matches = No hay coincidencias
main-empty-nothing-matches-body = Prueba con menos palabras o filtros como kind:película, site:github.com o #etiqueta.
main-empty-trash = La Papelera está vacía
main-empty-trash-body = Los marcadores que elimines esperan aquí durante 30 días.
main-empty-caught-up = Todo al día
main-empty-caught-up-body = Todos los marcadores de aquí ya están organizados.
main-empty-no-pictures = No hay imágenes aquí
main-empty-no-pictures-body = Los enlaces directos a un archivo de imagen aparecen aquí.
main-empty-no-events = No hay eventos aquí
main-empty-no-events-body = Las páginas de conciertos, charlas y encuentros aparecen aquí por fecha.
main-empty-nothing-yet = Aún no hay nada aquí
main-empty-nothing-yet-synced-body = Guarda un marcador en tu navegador y aparecerá aquí un momento después.
main-empty-nothing-yet-unsynced-body = Agrega un enlace con ⌘N, o elige un navegador para este espacio desde el menú de espacios.
main-empty-nothing-yet-body = Los marcadores aparecerán aquí a medida que los agregues.
main-up-next = A continuación
main-up-next-count =
    { $count ->
        [one] · { $count } para leer y ver
       *[other] · { $count } para leer y ver
    }
main-best-matches = Mejores resultados
main-place-gallery = Galería
main-place-events = Eventos
main-place-up-next = A continuación
main-place-unsorted = Sin carpeta
main-place-not-organized = Sin organizar
main-place-set-aside = Apartado
main-place-trash = Papelera
main-search-found =
    { $count ->
        [one] { $count } resultado
       *[other] { $count } resultados
    }
main-trash-empty = Vaciar Papelera
main-trash-delete-count =
    { $count ->
        [one] Eliminar { $count } para siempre
       *[other] Eliminar { $count } para siempre
    }
main-trash-about = Los marcadores en la Papelera se eliminan para siempre después de 30 días.
main-trash-days-left =
    { $days ->
        [1] En la Papelera. Se eliminará para siempre en menos de un día.
       *[other] En la Papelera. Se eliminará para siempre en { $days } días.
    }
main-add-link-title = Agregar un enlace a { $workspace }
main-add-link-not-a-link = Eso no parece una dirección web.
main-add-link-add = Agregar
main-image-file-name = Imagen
main-viewer-position = { $position } de { $count }
main-viewer-open-original = Abrir original
main-viewer-close = Cerrar
main-viewer-image-copied = Imagen copiada
main-viewer-copy-failed = No se pudo copiar la imagen
main-viewer-image-saved = Imagen guardada
main-viewer-save-failed = No se pudo guardar la imagen
main-search-placeholder = Empieza a escribir para buscar
main-token-scope = en { $place }
main-token-text = «{ $text }»
main-token-field = { $field }: { $value }
main-token-field-more = { $field } > { $value }
main-token-field-less = { $field } < { $value }
main-token-clear-all = Quitar todo
main-suggest-search-for = Buscar «{ $text }»
main-suggest-kinds = Tipos
main-suggest-folders = Carpetas
main-suggest-tags = Etiquetas
main-suggest-sites = Sitios
main-suggest-fields = Detalles
main-suggest-bookmarks = Marcadores
main-suggest-field = { $field }: { $value }
main-suggest-footer = ↑↓ mover · ↵ aplicar · ⇥ completar · ⌫ quitar el último
main-refine-narrow = Afinar
main-refine-more = Más
main-refine-clear = Borrar
main-display = Visualización
main-display-summaries = Mostrar resúmenes en las tarjetas
main-display-up-next = Mostrar la franja de A continuación
main-duration-minutes = { $minutes } min
main-duration-hours = { $hours } h
main-duration-hours-minutes = { $hours } h { $minutes } min

## Detail

detail-details = Detalles
detail-removed-from-browser = Eliminado del navegador, conservado aquí
detail-saved = Guardado el { $date }
detail-chosen-by = Indica lo que eligió { $model }
detail-folders = Carpetas
detail-no-folders = Aún no hay carpetas. Crea una en la barra lateral.
detail-folder-undo = Deshacer: quitar de aquí
detail-folder-take-out = Quitar de aquí
detail-folder-probability = { $percent }%
detail-add-to-another-folder = Agregar a otra carpeta
detail-set-aside = Apartado: no encaja en este espacio
detail-keep = Conservar
detail-organizing-now = Organizando este marcador ahora…
detail-organize-failed = No se pudo organizar.
detail-try-again = Intentar de nuevo
detail-title-and-link-only = Por ahora solo están el título y el enlace.
detail-read-without-ai = Leído solo de la página, sin IA.
detail-no-thumbnail = Sin miniatura en la página
detail-page-unreachable = No se pudo acceder a la página
detail-more = Más
detail-tags = Etiquetas
detail-remove-tag = Quitar etiqueta
detail-put-back = Volver a poner
detail-add-tag = Agregar etiqueta
detail-add-a-tag = Agregar una etiqueta
detail-edit = Editar título y descripción
detail-title = Título
detail-add-description = Agregar una descripción
detail-edit-hint = ↩ guardar · esc cancelar
detail-in-tag-list = En la lista de etiquetas
detail-in-tag-list-full = En la lista de etiquetas (lista llena)
detail-edit-tag-list = Editar lista de etiquetas…

## Events

events-title = Eventos
events-view-calendar = Calendario
events-view-list = Lista
events-previous-month = Mes anterior
events-next-month = Mes siguiente
events-today = Hoy
events-upcoming = Próximos
events-happening-now = En curso
events-date-not-set = Sin fecha
events-past = Pasados
events-nothing-on-day = Nada en este día.
events-nothing-coming-up = Nada próximamente.
events-nothing-coming-up-panel = Nada próximamente. Los eventos que guardes aparecerán aquí hasta que terminen.
events-more = +{ $count } más

## Dates

date-today = Hoy
date-yesterday = Ayer
date-tomorrow = Mañana
date-this-week = Esta semana
date-last-week = La semana pasada
date-all-day = Todo el día

## Kinds

kind-article = Artículo
kind-article-plural = Artículos
kind-video = Video
kind-video-plural = Videos
kind-movie = Película
kind-movie-plural = Películas
kind-recipe = Receta
kind-recipe-plural = Recetas
kind-product = Producto
kind-product-plural = Productos
kind-repository = Repositorio
kind-repository-plural = Código
kind-book = Libro
kind-book-plural = Libros
kind-image = Imagen
kind-image-plural = Imágenes
kind-app = App
kind-app-plural = Apps
kind-podcast = Podcast
kind-podcast-plural = Podcasts
kind-place = Lugar
kind-place-plural = Lugares
kind-event = Evento
kind-event-plural = Eventos
kind-other = Otro
kind-other-plural = Otros

## Fields

field-address = Dirección
field-attendance = Asistencia
field-author = Autor
field-brand = Marca
field-cast = Reparto
field-category = Categoría
field-channel = Canal
field-city = Ciudad
field-colors = Colores
field-country = País
field-cuisine = Cocina
field-currency = Moneda
field-diet = Dieta
field-director = Dirección
field-duration = Duración
field-ends = Fin
field-format = Formato
field-genre = Género
field-host = Presentador
field-language = Idioma
field-license = Licencia
field-maker = Creador
field-medium = Medio
field-name = Nombre
field-organizer = Organizador
field-owner = Propietario
field-pages = Páginas
field-platform = Plataforma
field-price = Precio
field-pricing = Precio
field-published = Publicado
field-publisher = Editorial
field-rating = Puntuación
field-reading-minutes = Tiempo de lectura
field-resolution = Resolución
field-runtime = Duración
field-servings = Porciones
field-show = Programa
field-stars = Estrellas
field-starts = Inicio
field-total-time = Tiempo total
field-venue = Lugar
field-visible-text = Texto visible
field-year = Año

## Inks

ink-ochre = Ocre
ink-olive = Oliva
ink-moss = Musgo
ink-pine = Pino
ink-teal = Verde azulado
ink-sky = Cielo
ink-cobalt = Cobalto
ink-violet = Violeta
ink-plum = Ciruela
ink-rose = Rosa
ink-cocoa = Cacao
ink-slate = Pizarra

## Errors

error-cant-start = { $name } no puede iniciarse
error-quit = Salir
error-signed-out = Se cerró tu sesión. Vuelve a iniciar sesión para seguir sincronizando.
error-sync-unanswered = El servidor no aceptó los últimos cambios.
error-unlink-library = No se pudo desvincular la biblioteca: { $error }
error-mcp-load = No se pudo cargar el acceso para IA: { $problem }
error-mcp-save = No se pudo guardar el acceso para IA: { $problem }
error-mcp-needs-pro = El acceso para IA requiere { -knoa } Pro.
