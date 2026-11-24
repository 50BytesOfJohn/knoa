## Terms
## One word per concept, used everywhere in this language.

-knoa = Knoa
-workspace = espace
-bookmark = signet
-folder = dossier
-tag = tag
-summary = résumé
-organize = organiser

## Menus

menu-about = À propos de { $name }
menu-check-for-updates = Rechercher les mises à jour…
menu-settings = Réglages…
menu-services = Services
menu-hide = Masquer { $name }
menu-hide-others = Masquer les autres
menu-show-all = Tout afficher
menu-quit = Quitter { $name }
menu-help = Aide
menu-whats-new = Nouveautés de { $name } { $version }
menu-report-issue = Signaler un problème…

## Onboarding

onboarding-skip-setup = Passer la configuration →
onboarding-back = Retour
onboarding-next = Suivant
onboarding-hello-title = Bonjour, je suis { -knoa }.
onboarding-hello-lead = Vous accumulez les signets comme un écureuil fait ses provisions pour l’hiver. Moi, je les retrouve. Je lis les signets de votre navigateur, j’identifie chacun d’eux et je garde tout bien rangé.
onboarding-hello-questions = Trois petites questions. Ou aucune, si vous êtes pressé.
onboarding-hello-start = C’est parti
onboarding-hello-defaults = Utiliser les réglages par défaut
onboarding-hello-have-account = Vous avez déjà un compte { -knoa } ?
onboarding-hello-sign-in = Se connecter
onboarding-browser-title = Où sont rangés vos signets ?
onboarding-browser-lead = Continuez d’enregistrer vos signets comme d’habitude. Je repère les nouveaux en quelques secondes. Je me contente de les lire ; je ne touche jamais aux signets de votre navigateur.
onboarding-browser-count =
    { $count ->
        [one] { $count } signet
       *[other] { $count } signets
    }
onboarding-browser-none-found = Je n’ai trouvé aucun navigateur basé sur Chromium ici (Helium, Chrome, Brave, Edge, Vivaldi). Pas de souci : vous pourrez en choisir un plus tard dans les Réglages.
onboarding-browser-bring-in = En importer { $count }
onboarding-ai-title = Vous voulez que je fasse le rangement ?
onboarding-ai-lead = Avec une clé OpenRouter, je lis chaque nouvelle page et lui attribue un type, un résumé, des tags et un dossier. C’est votre clé et votre compte : environ 0,17 $ pour mille signets.
onboarding-ai-key-placeholder = Collez votre clé : sk-or-v1-…
onboarding-ai-check-key = Vérifier la clé
onboarding-ai-checking = Je frappe à la porte d’OpenRouter…
onboarding-ai-key-works = Cette clé fonctionne.
onboarding-ai-key-missing = Collez d’abord une clé, ou passez cette étape.
onboarding-ai-key-unknown = OpenRouter ne reconnaît pas cette clé. Vous voulez la revérifier ?
onboarding-ai-key-check-failed = Impossible de vérifier la clé : { $error }
onboarding-ai-get-key = Obtenir une clé ↗
onboarding-ai-without-key = Sans clé, je récupère quand même les images et les descriptions.
onboarding-ai-skip = Passer, sans IA
onboarding-library-title = Dernière chose : comment s’appelle cette bibliothèque ?
onboarding-library-default-name = Personnel
onboarding-library-house-rules = Des règles maison pour l’organisation ?
onboarding-library-instructions-placeholder = Facultatif. Écrivez comme vous le diriez à un ami.
onboarding-library-example-apart = Séparer le travail et le perso.
onboarding-library-example-recipes = Tagger les recettes par type de cuisine.
onboarding-library-example-rust = Tout ce qui parle de Rust va dans un dossier Rust.
onboarding-library-example-short = Résumés courts, sans blabla.
onboarding-library-add-example = + { $example }
onboarding-library-finish = Terminer
onboarding-done-title = Tout est prêt.
onboarding-done-no-browser = Aucun navigateur pour l’instant, votre bibliothèque démarre donc vide. Choisissez-en un dans les Réglages quand vous voulez.
onboarding-done-bringing-in = Importation de vos signets…
onboarding-done-count =
    { $count ->
        [one] { $count } signet importé, prêt pour la recherche.
       *[other] { $count } signets importés, prêts pour la recherche.
    }
onboarding-done-organize-existing =
    { $count ->
        [one] Les nouveaux signets s’organisent tout seuls. Voulez-vous que je m’occupe aussi de celui que vous avez déjà ? Environ { $cost } $.
       *[other] Les nouveaux signets s’organisent tout seuls. Voulez-vous que je m’occupe aussi des { $count } que vous avez déjà ? Environ { $cost } $.
    }
onboarding-done-organize = Oui, les organiser
onboarding-done-open-library = Ouvrir ma bibliothèque

## Settings

settings-title = Réglages
settings-workspaces = Espaces
settings-new-workspace = Nouvel espace
settings-nav-general = Général
settings-nav-appearance = Apparence
settings-nav-ai = IA
settings-nav-account = Compte
settings-nav-organize = Organiser
settings-nav-mcp = MCP
settings-general-title = Général
settings-general-about = Où { -knoa } range votre bibliothèque, comment elle a été configurée et comment elle se met à jour.
settings-general-library = Bibliothèque
settings-general-location = Emplacement de la bibliothèque
settings-general-show-in-finder = Afficher dans le Finder
settings-general-setup = Configuration
settings-general-run-setup-again = Relancer la configuration
settings-general-run-setup-about = Choisissez à nouveau vos navigateurs, l’IA et vos instructions. Vos signets restent intacts.
settings-general-run-setup = Lancer la configuration
settings-general-language = Langue
settings-general-language-label = Langue de l’app et de l’IA
settings-general-language-about = Les menus, réglages et dates, ainsi que les résumés et tags rédigés par l’IA.
settings-general-language-note = Les résumés et tags déjà rédigés restent dans leur ancienne langue. Réorganiser votre bibliothèque les réécrit.
settings-general-language-organize = Réorganiser…
settings-general-language-organize-confirm = Tout réorganiser
settings-general-language-waiting = Vous pourrez réorganiser une fois que votre compte aura la nouvelle langue.
settings-updates = Mises à jour
settings-updates-version = { -knoa } { $version }
settings-updates-self-built = Cette version ne se met pas à jour automatiquement. Les versions publiées sur GitHub le font.
settings-updates-checked-today = Dernière vérification aujourd’hui à { $time }
settings-updates-checked-yesterday = Dernière vérification hier à { $time }
settings-updates-checked-on = Dernière vérification le { $date }
settings-updates-not-checked = Pas encore vérifié
settings-updates-check-now = Vérifier maintenant
settings-updates-check-automatically = Rechercher les mises à jour automatiquement
settings-updates-check-automatically-about = Une fois par jour, en arrière-plan.
settings-updates-install-automatically = Installer les mises à jour automatiquement
settings-updates-install-automatically-about = Télécharger les nouvelles versions et les installer lorsque vous quittez { -knoa }.
settings-appearance-title = Apparence
settings-appearance-about = L’aspect de { -knoa }. Les changements s’appliquent aussitôt.
settings-appearance-theme = Thème
settings-appearance-light = Clair
settings-appearance-dark = Sombre
settings-appearance-library = Bibliothèque
settings-appearance-summaries = Afficher les résumés sur les cartes
settings-appearance-summaries-about = Jusqu’à trois lignes sous chaque titre.
settings-appearance-up-next = Afficher le bandeau « À suivre »
settings-appearance-up-next-about = La rangée d’éléments à lire ou à regarder, en haut de « Tout ».
settings-appearance-events = Affichage des événements
settings-appearance-events-about = Un calendrier mensuel avec ce qui arrive, ou une seule liste. Sur ce Mac uniquement.
settings-reorganize-title = Réorganiser
settings-reorganize-folders-confirm = Tout classer
settings-reorganize-everything-confirm = Tout réorganiser
settings-organize-title = Organiser
settings-organize-about = Demandez à l’IA d’organiser les signets qu’elle n’a pas encore traités, ou de repasser sur ceux qu’elle a déjà faits.
settings-organize-scope = Signets dans
settings-organize-scope-all = Tous les espaces
settings-organize-new = Pas encore organisés
settings-organize-counting = Comptage des signets…
settings-organize-none = Tout est organisé ici.
settings-organize-waiting =
    { $count ->
        [one] { $count } signet n’est pas encore organisé
       *[other] { $count } signets ne sont pas encore organisés
    }
settings-organize-ai-off = L’organisation nécessite l’IA
settings-organize-ai-off-about = Connectez votre propre clé ou passez à Pro sur la page IA.
settings-organize-open-ai = Configurer l’IA
settings-reorganize-folders-label = Dossiers seulement
settings-reorganize-folders-about = Range les signets organisés dans vos dossiers actuels. Les pages ne sont pas relues, c’est donc rapide et peu coûteux.
settings-reorganize-folders-start = Reclasser…
settings-reorganize-everything-label = Tout
settings-reorganize-everything-about = Relit chaque page et refait son résumé, ses champs, ses tags et ses dossiers avec les instructions et la langue actuelles.
settings-reorganize-everything-start = Réorganiser…
settings-reorganize-kept = Les dossiers que vous avez définis ou retirés vous-même restent tels quels.
settings-reorganize-queued =
    { $count ->
        [one] { $count } signet mis en file d’attente.
       *[other] { $count } signets mis en file d’attente.
    }
settings-account-synced-today = Dernière synchronisation aujourd’hui à { $time }
settings-account-synced-yesterday = Dernière synchronisation hier à { $time }
settings-account-synced-on = Dernière synchronisation le { $date }
settings-account-not-synced = Pas encore synchronisé
settings-account-start-trial = Démarrer l’essai gratuit
settings-account-upgrade = Passer à Pro
settings-account-manage-subscription = Gérer l’abonnement
settings-account-usage = Utilisation
settings-account-usage-device = { $used } sur { $limit } signets
settings-account-usage-cloud = { $used } sur { $limit } signets, { $organized } sur { $allowance } organisés ce mois-ci
settings-account-plan-free = Offre gratuite.
settings-account-plan-trial = Essai Pro.
settings-account-plan-pro = Offre Pro.
settings-account-plan-ends = Se termine le { $date }.
settings-account-plan-trial-ends = L’essai se termine le { $date }.
settings-account-plan-renews = Renouvellement le { $date }.
settings-account-organized-on-device = L’organisation utilise votre propre clé IA, sur ce Mac.
settings-account-organized-in-cloud = { -knoa } organise vos signets dans le cloud.
settings-account-title = Compte
settings-account-about = Synchronisez votre bibliothèque avec un compte { -knoa }.
settings-account-unavailable = La synchronisation cloud n’est pas disponible dans cette version.
settings-account-couldnt-start = Impossible de démarrer : { $reason }
settings-account-cloud-mode = Mode cloud
settings-account-what-it-is = Le mode cloud synchronise votre bibliothèque sur tous vos appareils et avec { -knoa } sur le web. Avec Pro, { -knoa } organise aussi vos signets dans le cloud, sans clé personnelle.
settings-account-privacy = En mode cloud, votre bibliothèque est hébergée sur les serveurs de { -knoa }, qui lisent vos pages pour les organiser. Elle n’est pas chiffrée de bout en bout. Déconnecté, tout reste sur ce Mac.
settings-account-finish-signing-in = Terminez la connexion dans votre navigateur…
settings-account-finish-signing-in-about = { -knoa } a ouvert la page de connexion dans votre navigateur.
settings-account-cancel = Annuler
settings-account-knoa-account = Compte { -knoa }
settings-account-sign-in-about = Connectez-vous ou créez un compte dans votre navigateur.
settings-account-sign-in = Se connecter
settings-account-other-account = Cette bibliothèque est synchronisée avec un autre compte { -knoa }. Rien ne se synchronise tant que vous n’avez pas choisi.
settings-account-sync-with = Synchroniser plutôt avec { $email }
settings-account-sync-with-this-account = Synchroniser plutôt avec ce compte
settings-account-sync-with-about = La bibliothèque est fusionnée dans ce compte. L’autre compte conserve sa copie.
settings-account-use-this-account = Utiliser ce compte
settings-account-this-account = Ce compte
settings-account-asking = Connecté. Vérification du compte auprès du serveur…
settings-account-sync = Synchronisation
settings-account-connecting = Connexion…
settings-account-up-to-date = À jour
settings-account-syncing =
    { $count ->
        [one] Synchronisation de { $count } modification…
       *[other] Synchronisation de { $count } modifications…
    }
settings-account-offline = Hors ligne. Les modifications seront synchronisées dès que { -knoa } pourra joindre le serveur.
settings-account-sync-stopped = Synchronisation arrêtée : { $error }
settings-account-notice-plan-limit =
    { $unsent ->
        [one] Votre offre inclut { $limit } signets ; { $unsent } nouveau signet est donc resté sur ce Mac uniquement.
       *[other] Votre offre inclut { $limit } signets ; { $unsent } nouveaux signets sont donc restés sur ce Mac uniquement.
    }
settings-account-notice-rejected = Certaines modifications n’ont pas pu être synchronisées et sont restées sur ce Mac uniquement.
settings-account-notice-too-large = Certaines modifications étaient trop volumineuses pour être synchronisées et sont restées sur ce Mac uniquement.
settings-account-notice-allowance-used-up = Le quota d’organisation de ce mois-ci est épuisé. { -knoa } conserve vos demandes d’organisation et les reprend automatiquement dès que du quota redevient disponible.
settings-account-sign-out = Se déconnecter
settings-account-sign-out-about = Votre bibliothèque reste sur ce Mac. Reconnectez-vous pour reprendre la synchronisation.
settings-ai-access-about = Permettez à des apps IA comme Claude et ChatGPT de lire et gérer vos signets.
settings-ai-access-copy = Copier
settings-ai-access-url = URL du connecteur
settings-ai-access-default = Accès par défaut
settings-ai-access-same-as-default = Identique au réglage par défaut
settings-ai-access-off = Désactivé
settings-ai-access-read = Lecture seule
settings-ai-access-full = Accès complet
settings-ai-access-not-linked = Disponible une fois cet espace lié au cloud.
settings-ai-access-saving = Enregistrement…
settings-ai-access-loading = Chargement de l’accès IA…
settings-ai-access-save-again = Réenregistrer l’accès IA
settings-ai-access-load-again = Recharger l’accès IA
settings-ai-access-retry = Réessayer
settings-ai-access-needs-pro = L’accès IA nécessite { -knoa } Pro
settings-mcp-title = MCP
settings-mcp-sign-in = Connectez-vous pour relier des apps IA
settings-mcp-sign-in-about = Les apps IA accèdent à votre bibliothèque via votre compte { -knoa }.
settings-mcp-open-account = Ouvrir Compte
settings-mcp-pro-about = Avec Pro, des apps IA comme Claude et ChatGPT peuvent rechercher, lire et organiser votre bibliothèque.
settings-mcp-copied = Copié
settings-mcp-connect = Connexion
settings-mcp-how = Comment se connecter
settings-mcp-how-copy = Copiez l’URL du connecteur.
settings-mcp-how-add = Dans Claude, ChatGPT ou une autre app IA, ajoutez-la comme connecteur personnalisé.
settings-mcp-how-sign-in = Connectez-vous à { -knoa } lorsque l’app vous le demande.
settings-mcp-how-about = Une app IA connectée peut lire vos signets, y compris le texte enregistré depuis leurs pages, dans chaque espace auquel elle a accès. Les changements ici s’appliquent dès sa prochaine requête.
settings-mcp-access = Accès
settings-mcp-default-off = Les apps IA ne peuvent pas accéder aux espaces qui utilisent le réglage par défaut.
settings-mcp-default-read = Les apps IA peuvent rechercher et lire les signets dans les espaces qui utilisent le réglage par défaut.
settings-mcp-default-full = Les apps IA peuvent aussi ajouter, modifier et placer des signets dans la Corbeille, ainsi que gérer les dossiers et les espaces.
settings-ai-title = IA
settings-ai-about = { -knoa } fonctionne aussi sans IA. Avec elle, les nouveaux signets reçoivent un type, un résumé, des tags et des dossiers.
settings-ai-provider = Fournisseur
settings-ai-cloud-note = Avec Pro, { -knoa } organise vos signets dans le cloud. La clé que vous avez connectée reste sur ce Mac pour quand vous êtes déconnecté.
settings-ai-key = Clé OpenRouter
settings-ai-key-about = Votre propre clé ; les requêtes partent de ce Mac directement vers OpenRouter.
settings-ai-connect = Connecter
settings-ai-checking = Vérification de la clé…
settings-ai-key-connected = Connectée. Les nouveaux signets sont désormais organisés.
settings-ai-key-failed = L’opération a échoué : { $error }.
settings-ai-no-key = Pas encore de clé ?
settings-ai-no-key-about = Quelques dollars de crédit suffisent pour organiser des milliers de signets.
settings-ai-get-key = Obtenir une clé
settings-ai-connected = Connecté
settings-ai-disconnect = Déconnecter
settings-ai-openrouter-says = OpenRouter indique : { $problem }. Vérifiez votre clé ou vos crédits.
settings-ai-models = Modèles
settings-ai-included-in-pro = Inclus avec Pro, dans la limite du quota mensuel.
settings-ai-cost = Environ { $cost } $ pour tous les organiser.
settings-ai-organize = Organiser
settings-ai-pro-title = Laissez { -knoa } s’en charger avec Pro
settings-ai-pro-pitch = { -knoa } organise vos signets dans le cloud. Aucune clé à configurer, aucune facturation à la requête, et votre bibliothèque se synchronise sur tous vos appareils.
settings-ai-pro-get = Passer à Pro
settings-ai-pro-upgrade = Passer à Pro
settings-ai-pro-have = J’ai déjà Pro
settings-ai-usage = Utilisation
settings-ai-usage-none = Aucune requête IA pour l’instant.
settings-ai-usage-recent =
    { $days ->
        [one] Dernier jour
       *[other] { $days } derniers jours
    }
settings-ai-usage-all-time = Depuis le début
settings-ai-usage-by-model =
    { $days ->
        [one] Par modèle, dernier jour
       *[other] Par modèle, { $days } derniers jours
    }
settings-ai-usage-records = Relevés de { $provider }
settings-ai-usage-records-about = Chaque requête, avec le montant exact facturé.
settings-ai-usage-open = Ouvrir
settings-ai-usage-requests =
    { $count ->
        [one] { $count } requête
       *[other] { $count } requêtes
    }
settings-ai-usage-tokens = { $input } tokens en entrée, { $output } en sortie
settings-ai-usage-unmetered =
    { $count ->
        [one] { $count } sans décompte de tokens
       *[other] { $count } sans décompte de tokens
    }
settings-ai-usage-unpriced =
    { $count ->
        [one] { $count } sans tarif
       *[other] { $count } sans tarif
    }
settings-workspace-new = Nouvel espace
settings-workspace-new-about = Une bibliothèque distincte, comme Travail ou Personnel, avec ses propres navigateurs et dossiers.
settings-workspace-shown-now = Affiché actuellement
settings-workspace-show-it = Afficher
settings-workspace-name-and-color = Nom et couleur
settings-workspace-name = Nom
settings-workspace-name-too-long = Un nom peut contenir au maximum { $max } caractères.
settings-workspace-color = Couleur
settings-workspace-no-browser = Aucun navigateur basé sur Chromium trouvé. { -knoa } lit Helium, Chrome, Brave, Edge, Vivaldi et Chromium.
settings-workspace-browsers-note = Si aucun n’est activé, les liens ne sont ajoutés que manuellement (⌘N). Un navigateur peut alimenter plusieurs espaces.
settings-workspace-open-in = Ouvrir les signets dans
settings-workspace-browsers = Navigateurs
settings-workspace-open-in-about = Où s’ouvrent les liens de cet espace.
settings-workspace-organize-again-about = Après avoir changé les instructions ou les tags, reclassez ou réorganisez les signets de cet espace.
settings-workspace-open-organize = Organiser…
settings-workspace-default-browser = Navigateur par défaut
settings-workspace-by-hand = Liens ajoutés uniquement à la main.
settings-workspace-synced-from-one = Synchronisé depuis { $browser }.
settings-workspace-synced-from-many = Synchronisé depuis { $browsers } et { $last }.
settings-workspace-organizing = Organisation
settings-workspace-instructions = Instructions
settings-workspace-instructions-about = Comment l’IA doit classer, tagger et résumer ce qui arrive ici.
settings-workspace-instructions-too-long = Les instructions peuvent contenir au maximum { $max } caractères.
settings-workspace-screening = Ne garder que les signets qui correspondent
settings-workspace-screening-about = Désactivé : tout ce qui vient du navigateur arrive ici. Activé : l’IA met de côté ce qui ne correspond pas, et vous pouvez quand même le conserver.
settings-workspace-guideline = Ce qui y a sa place
settings-workspace-guideline-too-long = Une consigne peut contenir au maximum { $max } caractères.
settings-workspace-name-placeholder = Travail, Personnel…
settings-workspace-instructions-placeholder = Par ex. Tagger les recettes par type de cuisine. Résumés courts, sans blabla.
settings-workspace-guideline-placeholder = Par ex. Uniquement le travail : ingénierie, design et nos clients.
settings-workspace-create = Créer l’espace
settings-workspace-delete = Supprimer
settings-workspace-delete-label = Supprimer cet espace
settings-workspace-delete-about = Ses signets et ses dossiers sont supprimés avec lui. Vos navigateurs conservent leurs signets.
settings-workspace-delete-button = Supprimer…
settings-workspace-delete-confirm = Supprimer définitivement
settings-tags-title = Liste de tags
settings-tags-label = Tags que l’IA doit utiliser
settings-tags-about = Lorsque l’un d’eux correspond à un signet, l’IA l’utilise plutôt que d’inventer son propre mot. Une courte précision lui indique quand l’employer. Vous pouvez aussi faire un clic droit sur un tag d’un signet pour l’ajouter à la liste.
settings-tags-add-placeholder = Ajouter un tag
settings-tags-hint-placeholder = Quand l’utiliser (facultatif)
settings-tags-remove = Retirer de la liste
settings-tags-only-listed = Utiliser uniquement les tags de cette liste
settings-tags-only-listed-about = Désactivé : l’IA ajoute ses propres tags lorsque la liste ne suffit pas. Activé : les signets ne reçoivent que des tags de la liste, ou aucun si rien ne correspond.
settings-tags-only-listed-about-empty = Ajoutez d’abord un tag. Désactivé : l’IA ajoute ses propres tags lorsque la liste ne suffit pas. Activé : les signets ne reçoivent que des tags de la liste, ou aucun si rien ne correspond.
settings-tags-note = Les signets organisés à partir de maintenant reçoivent ces tags. Pour les appliquer à ceux déjà organisés, réorganisez.
settings-tags-count = { $count } sur { $max }

## Sidebar

sidebar-status-ai-needs-attention = L’IA a besoin d’attention
sidebar-status-fix = Corriger
sidebar-sync-stopped = Synchronisation arrêtée
sidebar-retry = Réessayer
sidebar-sync-paused = Synchronisation en pause
sidebar-status-changes-not-synced = Modifications non synchronisées
sidebar-status-view = Voir
sidebar-status-plan-limit = Limite de l’offre atteinte
sidebar-status-upgrade = Passer à Pro
sidebar-status-organizing = Organisation : { $current } sur { $total }…
sidebar-status-ai-off = IA désactivée
sidebar-status-turn-on = Activer
sidebar-status-offline = Hors ligne. Synchronisation différée.
sidebar-syncing = Synchronisation…
sidebar-sync-offline =
    { $count ->
        [0] Hors ligne
        [one] Hors ligne · { $count } modification en attente
       *[other] Hors ligne · { $count } modifications en attente
    }
sidebar-synced-just-now = Synchronisé à l’instant
sidebar-synced-minutes-ago =
    { $minutes ->
        [one] Synchronisé il y a { $minutes } min
       *[other] Synchronisé il y a { $minutes } min
    }
sidebar-synced-hours-ago =
    { $hours ->
        [one] Synchronisé il y a { $hours } h
       *[other] Synchronisé il y a { $hours } h
    }
sidebar-account-settings = Réglages du compte…
sidebar-open-web = Ouvrir { -knoa } sur le web
sidebar-sign-out = Se déconnecter
sidebar-sign-in = Se connecter…
sidebar-finish-in-browser = Terminer dans le navigateur
sidebar-cancel = Annuler
sidebar-plan-pro = { -knoa } Pro
sidebar-plan-free = Offre gratuite
sidebar-signed-in = Connecté
sidebar-manage-plan = Gérer l’offre…
sidebar-upgrade-to-pro = Passer à Pro
sidebar-this-mac = Ce Mac
sidebar-knoa-account = Compte { -knoa }
sidebar-place-everything = Tout
sidebar-place-gallery = Galerie
sidebar-place-events = Événements
sidebar-place-up-next = À suivre
sidebar-place-unsorted = Sans dossier
sidebar-place-not-organized = Non organisé
sidebar-place-set-aside = Mis de côté
sidebar-place-trash = Corbeille
sidebar-settings = Réglages
sidebar-folders = Dossiers
sidebar-new-folder = Nouveau dossier
sidebar-folder-new-subfolder = Nouveau sous-dossier
sidebar-folder-edit = Modifier le dossier…
sidebar-folder-delete = Supprimer…
sidebar-folder-move-to = Déplacer vers
sidebar-folder-top-level = Niveau supérieur
sidebar-add-link = Ajouter un lien
sidebar-workspaces-title = Espaces
sidebar-workspaces-new = Nouvel espace
sidebar-workspaces-by-hand = Liens ajoutés manuellement
sidebar-workspaces-from = Depuis { $browsers }

## Main

main-blocked-browsers =
    { $count ->
        [one] { $browsers } est installé, mais macOS protège ses signets. Accordez l’Accès complet au disque à { -knoa }, puis redémarrez { -knoa }.
       *[other] { $browsers } sont installés, mais macOS protège leurs signets. Accordez l’Accès complet au disque à { -knoa }, puis redémarrez { -knoa }.
    }
main-open-privacy-settings = Ouvrir les réglages Confidentialité
main-cancel = Annuler
main-delete = Supprimer
main-save = Enregistrer
main-folder-new-title = Nouveau dossier
main-folder-edit-title = Modifier le dossier
main-folder-name-placeholder = Nom du dossier
main-folder-description-placeholder = Qu’est-ce qui a sa place ici ? L’IA s’en sert pour classer les signets.
main-folder-name-too-long = Un nom peut contenir au maximum { $max } caractères.
main-folder-description-too-long = Une description peut contenir au maximum { $max } caractères.
main-folder-auto-assign = Laisser l’IA ajouter des signets
main-folder-auto-assign-about = Les nouveaux signets qui correspondent y vont automatiquement.
main-glyph-read = Lecture
main-glyph-article = Articles
main-glyph-code = Code
main-glyph-write = Écriture
main-glyph-work = Travail
main-glyph-tag = Bons plans
main-glyph-bag = Achats
main-glyph-plane = Voyages
main-glyph-cup = Cuisine
main-glyph-music = Musique
main-glyph-play = Vidéo
main-glyph-photo = Photos
main-glyph-bulb = Idées
main-glyph-heart = Santé
main-glyph-home = Maison
main-glyph-star = Favoris
main-glyph-clock = Plus tard
main-glyph-flask = Sciences
main-glyph-globe = Web
main-glyph-person = Personnes
main-glyph-box = Archives
main-glyph-inbox = Boîte de réception
main-glyph-leaf = Nature
main-glyph-bolt = Rapide
main-glyph-lock = Privé
main-glyph-pin = Lieux
main-glyph-chat = Discussions
main-glyph-game = Jeux
main-folder-icon-none = Dossier
main-folder-icon-emoji = Émoji
main-folder-parent = Parent
main-folder-parent-none = Aucun
main-folder-emoji-placeholder = Saisir ou coller
main-folder-any-emoji = N’importe quel émoji
main-folder-custom-color = Personnalisée
main-delete-folder-title = Supprimer « { $name } » ?
main-delete-folder-detail =
    { $count ->
        [0] Ses signets restent dans votre bibliothèque.
        [one] Son sous-dossier est supprimé avec lui. Leurs signets restent dans votre bibliothèque.
       *[other] Ses { $count } sous-dossiers sont supprimés avec lui. Leurs signets restent dans votre bibliothèque.
    }
main-open = Ouvrir
main-open-in-background = Ouvrir en arrière-plan
main-copy-link = Copier le lien
main-copied = Copié
main-show-details = Afficher les détails
main-view-picture = Voir l’image
main-copy-image = Copier l’image
main-save-image = Enregistrer l’image…
main-add-to-up-next = Ajouter à « À suivre »
main-remove-from-up-next = Retirer de « À suivre »
main-add-to-folder = Ajouter à un dossier
main-remove-from-folder = Retirer de { $folder }
main-refresh-thumbnail = Actualiser la vignette
main-refreshing-thumbnail = Actualisation de la vignette…
main-organize = Organiser
main-organize-again = Réorganiser
main-organizing = Organisation…
main-move-to-trash = Placer dans la Corbeille
main-restore = Restaurer
main-delete-now = Supprimer maintenant
main-open-link = Ouvrir le lien
main-card-failed = Impossible d’organiser
main-card-not-organized = Pas encore organisé
main-empty-no-events-match = Aucun événement trouvé
main-empty-no-events-match-body = Essayez avec moins de mots, ou un filtre de date comme starts>2026-12-01.
main-empty-nothing-matches = Aucun résultat
main-empty-nothing-matches-body = Essayez avec moins de mots, ou des filtres comme kind:film, site:github.com ou #tag.
main-empty-trash = La Corbeille est vide
main-empty-trash-body = Les signets que vous supprimez restent ici pendant 30 jours.
main-empty-caught-up = Tout est à jour
main-empty-caught-up-body = Tous les signets ici ont été organisés.
main-empty-no-pictures = Aucune image ici
main-empty-no-pictures-body = Les liens pointant directement vers un fichier image apparaissent ici.
main-empty-no-events = Aucun événement ici
main-empty-no-events-body = Les pages de concerts, conférences et rencontres apparaissent ici par date.
main-empty-nothing-yet = Rien ici pour l’instant
main-empty-nothing-yet-synced-body = Enregistrez un signet dans votre navigateur et il arrive ici un instant plus tard.
main-empty-nothing-yet-unsynced-body = Ajoutez un lien avec ⌘N, ou choisissez un navigateur pour cet espace dans le menu des espaces.
main-empty-nothing-yet-body = Les signets apparaissent ici à mesure que vous les ajoutez.
main-up-next = À suivre
main-up-next-count =
    { $count ->
        [one] · { $count } à lire ou regarder
       *[other] · { $count } à lire ou regarder
    }
main-best-matches = Meilleurs résultats
main-place-gallery = Galerie
main-place-events = Événements
main-place-up-next = À suivre
main-place-unsorted = Sans dossier
main-place-not-organized = Non organisé
main-place-set-aside = Mis de côté
main-place-trash = Corbeille
main-search-found =
    { $count ->
        [one] { $count } résultat
       *[other] { $count } résultats
    }
main-kind-all = Tout
main-kind-chip = { $kind } { $count }
main-trash-empty = Vider la Corbeille
main-trash-delete-count =
    { $count ->
        [one] Supprimer définitivement { $count } signet
       *[other] Supprimer définitivement { $count } signets
    }
main-trash-about = Les signets dans la Corbeille sont supprimés définitivement après 30 jours.
main-trash-days-left =
    { $days ->
        [1] Dans la Corbeille. Supprimé définitivement dans moins d’un jour.
       *[other] Dans la Corbeille. Supprimé définitivement dans { $days } jours.
    }
main-add-link-title = Ajouter un lien dans { $workspace }
main-add-link-not-a-link = Cela ne ressemble pas à une adresse web.
main-add-link-add = Ajouter
main-image-file-name = Image
main-viewer-position = { $position } sur { $count }
main-viewer-open-original = Ouvrir l’original
main-viewer-close = Fermer
main-viewer-image-copied = Image copiée
main-viewer-copy-failed = Impossible de copier l’image
main-viewer-image-saved = Image enregistrée
main-viewer-save-failed = Impossible d’enregistrer l’image
main-search-placeholder = Tapez simplement pour rechercher
main-token-scope = dans { $place }
main-token-text = « { $text } »
main-token-field = { $field } : { $value }
main-token-field-more = { $field } > { $value }
main-token-field-less = { $field } < { $value }
main-token-clear-all = Tout effacer
main-view-options-title = Affichage
main-view-options-summaries = Afficher les résumés sur les cartes
main-view-options-up-next = Afficher le bandeau « À suivre »
main-view-options-about = Les cartes sont groupées par date. La recherche classe par pertinence.
main-duration-minutes = { $minutes } min
main-duration-hours = { $hours } h
main-duration-hours-minutes = { $hours } h { $minutes } min

## Detail

detail-details = Détails
detail-removed-from-browser = Retiré du navigateur, conservé ici
detail-saved = Enregistré le { $date }
detail-chosen-by = Indique les choix de { $model }
detail-folders = Dossiers
detail-no-folders = Aucun dossier pour l’instant. Créez-en un dans la barre latérale.
detail-folder-undo = Annuler : retirer du dossier
detail-folder-take-out = Retirer du dossier
detail-folder-probability = { $percent } %
detail-add-to-another-folder = Ajouter à un autre dossier
detail-set-aside = Mis de côté : ne correspond pas à cet espace
detail-keep = Conserver
detail-organizing-now = Organisation en cours…
detail-organize-failed = L’organisation a échoué.
detail-try-again = Réessayer
detail-title-and-link-only = Seulement le titre et le lien pour l’instant.
detail-read-without-ai = Lu sur la page uniquement, sans IA.
detail-no-thumbnail = Aucune vignette sur la page
detail-page-unreachable = Impossible d’accéder à la page
detail-more = Plus
detail-tags = Tags
detail-remove-tag = Retirer le tag
detail-put-back = Remettre
detail-add-tag = Ajouter un tag
detail-add-a-tag = Ajouter un tag
detail-edit = Modifier le titre et la description
detail-title = Titre
detail-add-description = Ajouter une description
detail-edit-hint = ↩ enregistrer · échap annuler
detail-in-tag-list = Dans la liste de tags
detail-in-tag-list-full = Dans la liste de tags (liste pleine)
detail-edit-tag-list = Modifier la liste de tags…

## Events

events-title = Événements
events-view-calendar = Calendrier
events-view-list = Liste
events-previous-month = Mois précédent
events-next-month = Mois suivant
events-today = Aujourd’hui
events-upcoming = À venir
events-happening-now = En cours
events-date-not-set = Date non définie
events-past = Passés
events-nothing-on-day = Rien ce jour-là.
events-nothing-coming-up = Rien de prévu.
events-nothing-coming-up-panel = Rien de prévu. Les événements que vous enregistrez apparaissent ici jusqu’à leur fin.
events-more = +{ $count } de plus

## Dates

date-today = Aujourd’hui
date-yesterday = Hier
date-tomorrow = Demain
date-this-week = Cette semaine
date-last-week = La semaine dernière
date-all-day = Toute la journée

## Kinds

kind-article = Article
kind-article-plural = Articles
kind-video = Vidéo
kind-video-plural = Vidéos
kind-movie = Film
kind-movie-plural = Films
kind-recipe = Recette
kind-recipe-plural = Recettes
kind-product = Produit
kind-product-plural = Produits
kind-repository = Dépôt
kind-repository-plural = Code
kind-book = Livre
kind-book-plural = Livres
kind-image = Image
kind-image-plural = Images
kind-app = App
kind-app-plural = Apps
kind-podcast = Podcast
kind-podcast-plural = Podcasts
kind-place = Lieu
kind-place-plural = Lieux
kind-event = Événement
kind-event-plural = Événements
kind-other = Autre
kind-other-plural = Autres

## Fields

field-address = Adresse
field-attendance = Participation
field-author = Auteur
field-brand = Marque
field-cast = Distribution
field-category = Catégorie
field-channel = Chaîne
field-city = Ville
field-colors = Couleurs
field-country = Pays
field-cuisine = Cuisine
field-currency = Devise
field-diet = Régime
field-director = Réalisation
field-duration = Durée
field-ends = Fin
field-format = Format
field-genre = Genre
field-host = Animation
field-language = Langue
field-license = Licence
field-maker = Créateur
field-medium = Médium
field-name = Nom
field-organizer = Organisateur
field-owner = Propriétaire
field-pages = Pages
field-platform = Plateforme
field-price = Prix
field-pricing = Tarif
field-published = Publication
field-publisher = Éditeur
field-rating = Note
field-reading-minutes = Temps de lecture
field-resolution = Résolution
field-runtime = Durée
field-servings = Portions
field-show = Émission
field-stars = Étoiles
field-starts = Début
field-total-time = Temps total
field-venue = Lieu
field-visible-text = Texte visible
field-year = Année

## Inks

ink-ochre = Ocre
ink-olive = Olive
ink-moss = Mousse
ink-pine = Pin
ink-teal = Sarcelle
ink-sky = Ciel
ink-cobalt = Cobalt
ink-violet = Violet
ink-plum = Prune
ink-rose = Rose
ink-cocoa = Cacao
ink-slate = Ardoise

## Errors

error-cant-start = { $name } ne peut pas démarrer
error-quit = Quitter
error-signed-out = Vous avez été déconnecté. Reconnectez-vous pour continuer à synchroniser.
error-sync-unanswered = Le serveur n’a pas pris en compte les dernières modifications.
error-unlink-library = Impossible de dissocier la bibliothèque : { $error }
error-mcp-load = Impossible de charger l’accès IA : { $problem }
error-mcp-save = Impossible d’enregistrer l’accès IA : { $problem }
error-mcp-needs-pro = L’accès IA nécessite { -knoa } Pro.
