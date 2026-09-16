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

![Der Organisationsbaum mit ausgewählter Organisation und ihren Stammdaten](../images/settings-organization.png)

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

## Fachliche Vorgaben

Mit der Berechtigung `setting:read` zeigt die Detailansicht einer Organisation
zusätzlich den Abschnitt **Fachliche Vorgaben** mit zwei Werten: dem Wasserbedarf je
Baum in Litern und der Nachwirkzeit frisch gegossen in Stunden. Der Wasserbedarf geht
in die Bedarfsrechnung jeder Bewässerungsgruppe ein und bestimmt damit, wie viele
Gruppen eine Tourenplanung in eine Fahrt legt; eine spätere Änderung rechnet einen
bereits berechneten Einsatzplan nicht rückwirkend um. Die Nachwirkzeit bestimmt, wie
lange ein Baum nach einem abgeschlossenen Einsatzplan seiner Gruppe als
[Soeben bewässert](./treecluster.md#bewasserungsstatus-und-wie-er-zustande-kommt) gilt,
bevor er auf **Unbekannt** zurückfällt; einen aus Sensordaten berechneten
Bewässerungszustand zeigt er erst wieder, sobald tatsächlich neue Messwerte eintreffen.
Eine Änderung der Nachwirkzeit wirkt beim nächsten Lauf des dafür zuständigen
Hintergrundjobs, ohne dass die Anwendung dafür neu gestartet werden müsste.

Unter dem Namen jedes Werts steht, was er bewirkt, und daneben ein Abzeichen, das
seine Herkunft zeigt: **Vorgabe der Instanz** für den mitgelieferten Standardwert,
**Geerbt von …** mit dem Namen der Organisation, von der der Wert stammt, oder
**Eigener Wert**, wenn diese Organisation selbst einen gesetzt hat. Ohne eigenen Wert
übernimmt eine Organisation also automatisch, was ihre übergeordnete Organisation
vorgibt, den Baum hinauf bis zur obersten Organisation und von dort zur Vorgabe der
Instanz. Ein geerbter Wert steht schlicht als Zahl mit seiner Einheit da, denn es gibt
nichts einzutragen, solange er von woanders kommt; erst über **Eigenen Wert setzen**
wird daraus ein Eingabefeld, in das du eine eigene Zahl schreibst. **Wieder erben**
verwirft den eigenen Wert, und die Organisation folgt von da an erneut dem geerbten.
Ein senkrechter Strich links an jedem Wert greift dieselbe Unterscheidung auf: grün,
solange die Organisation den Wert selbst setzt, sonst grau. Darunter steht, wann und
von wem der Wert zuletzt geändert wurde, sofern das schon einmal geschehen ist; das
beantwortet die Frage, wer für eine überraschende Änderung verantwortlich ist.

![Der Abschnitt Fachliche Vorgaben mit einem geerbten und einem eigenen Wert](../images/settings-organization-values.png)

Der Schalter **Untereinheiten dürfen eigene Werte setzen** entscheidet, ob die
Organisationen unterhalb der gerade ausgewählten überhaupt eigene Werte setzen dürfen.
Steht er aus, sperrt die Organisation die Vorgaben für ihren gesamten Ast: Der
Abschnitt einer betroffenen Untereinheit wird dadurch nur noch lesbar, ein Hinweis oben
im Abschnitt nennt die sperrende Organisation, und es gelten dort deren Werte, nicht
mehr die eigenen. Hatte eine Untereinheit vor der Sperre bereits einen eigenen Wert
gesetzt, bleibt er sichtbar, unter dem Wert steht aber, dass er ruht, solange die
Sperre gilt; er kommt unverändert wieder zum Tragen, sobald sie wieder aufgehoben
wird. Der Satz unter dem Schalter beschreibt jeweils den Zustand, in dem er gerade
steht, damit die Tragweite des Ausschaltens nicht erst beim Ausprobieren auffällt.

Zum Ändern eines Werts oder des Schalters brauchst du zusätzlich zu `setting:read` die
Berechtigung `setting:update`. Ohne sie zeigt die Anwendung den Abschnitt nur an: alle
Werte stehen als Zahl da, **Eigenen Wert setzen** und **Wieder erben** fehlen, und ein
Hinweis oben im Abschnitt sagt, dass du die Vorgaben ansehen, aber nicht ändern kannst.
Wie die übrigen Stammdaten dieser Seite wirkt eine Änderung erst mit **Speichern**.
