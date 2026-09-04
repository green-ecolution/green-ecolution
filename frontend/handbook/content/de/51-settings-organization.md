---
slug: settings-organization
title: Organisation
part: administration
summary: Was eine Organisation im System bedeutet, wie der Organisationsbaum aufgebaut ist und wie Berechtigungen darin nach unten wirken.
routes: ['/settings/organization']
---

> [!NOTE]
> Diese Seite setzt die Berechtigung `organization:read` voraus. Fehlt sie dir, taucht
> **Organisation** in deiner Einstellungsnavigation gar nicht erst auf; wende dich an
> die Verwaltung deiner Organisation, wenn du sie brauchst.

## Der Organisationsbaum

Jede Organisation in Green Ecolution ist ein Knoten in einem Baum: Sie hat höchstens
eine übergeordnete Organisation und beliebig viele untergeordnete. An der Organisation
hängt, wem eine Person, ein Baum, eine Bewässerungsgruppe, ein Sensor, ein Fahrzeug
oder ein Einsatzplan gehört, und über welche Rollen Zugriff darauf vergeben werden
kann. Die Seite zeigt den Baum links als Liste zum Auf- und Zuklappen, rechts die
Stammdaten der ausgewählten Organisation; auf schmalen Bildschirmen öffnen sich die
Stammdaten stattdessen in einem eigenen Fenster.

Genau eine Organisation steht ganz oben im Baum, sie trägt instanzweit immer den Namen
Green Ecolution und lässt sich nicht bearbeiten oder löschen. Ein entsprechender
Hinweis erscheint, sobald du sie auswählst. Jede andere Organisation zeigt neben ihrem
Namen eine optionale Adresse, die nur vollständig gespeichert werden kann, eine
optionale Kontaktperson aus den ihr zugeordneten Mitarbeitenden sowie die Anzahl der
ihr direkt und über Unterorganisationen zugeordneten Personen. Mit der Berechtigung
`user:read` kommt zusätzlich eine Kachel **Zugewiesene Mitarbeitende** mit ihren
Kürzeln hinzu, die auf die Mitarbeitendenliste verlinkt.

## Unterorganisationen

Mit der Berechtigung `organization:create` legst du über **Unterorganisation anlegen**
eine neue Organisation unterhalb der gerade ausgewählten an; zunächst genügt ein Name,
Adresse und Kontaktperson trägst du anschließend in den Stammdaten nach. Der Name muss
sich von seinen Geschwistern unterscheiden, also von den anderen Organisationen mit
derselben übergeordneten Organisation; ein bereits vergebener Name wird zurückgewiesen.
Löschen lässt sich eine Organisation mit der Berechtigung `organization:delete` nur,
solange sie weder Unterorganisationen noch zugeordnete Mitarbeitende noch andere
Ressourcen wie Bäume oder Sensoren mehr besitzt, ein Hinweis unter **Organisation
löschen** erinnert daran.

Beim Anlegen bekommt eine neue Organisation sofort eigene, nutzbare Kopien aller
Rollenvorlagen; mehr dazu und wie sich diese Kopien zu eigenen Rollen entwickeln, steht
in [Team und Rollen](./settings-team.md#rollenvorlagen).

## Wie Berechtigungen im Baum wirken

Eine Rolle gehört immer zu genau einer Organisation und gilt für diese Organisation und
den gesamten Ast darunter, niemals für die Organisationen darüber. Ist eine Rolle also
einer weiter oben stehenden Organisation zugeordnet, reicht sie automatisch in jede ihr
untergeordnete Organisation hinein; eine Rolle einer untergeordneten Organisation
bleibt dagegen auf deren eigenen Ast beschränkt und erreicht weder Geschwister- noch
übergeordnete Organisationen. Deshalb liegt die Verwaltung, die über mehrere
Organisationen hinweg zuständig sein soll, sinnvollerweise weiter oben im Baum. Wie
Rollen im Einzelnen zugeschnitten werden und welche Rechte sich darin kombinieren
lassen, steht in [Team und Rollen](./settings-team.md).
