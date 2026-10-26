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

## Module

Green Ecolution besteht aus einem gemeinsamen Kern und Fachmodulen. Der Kern umfasst
die Organisationen, Rollen, Sensoren, Fahrzeuge und Startpunkte; ein Fachmodul bringt die
Bereiche mit, die zu einer bestimmten Aufgabe gehören. Das Modul **Bewässerung** liefert
Bäume, Bewässerungsgruppen, Einsatzpläne und die Auswertung samt der zugehörigen
Kartenebenen und Kacheln auf dem Dashboard.

Im Abschnitt **Module** der Organisationsseite steht für jedes Modul, das diese Instanz
kennt, eine Zeile mit einem Schalter. Ein eingeschaltetes Modul steht der ausgewählten
Organisation und allen Organisationen darunter zur Verfügung. Wie bei den
[Fachlichen Vorgaben](./settings-organization.md#fachliche-vorgaben) zeigt ein Abzeichen neben dem Schalter, woher
der Zustand kommt: ein eigener Wert dieser Organisation, ein von einer übergeordneten
Organisation geerbter Wert oder die Vorgabe der Instanz, solange noch keine Organisation
im Baum einen Wert gesetzt hat. Ohne eigenen Wert folgt eine Organisation also ihrer
übergeordneten. Den Schalter bedienst du direkt: Sobald du ihn umlegst, setzt die
Organisation einen eigenen Wert, und neben dem Schalter erscheint **Wieder erben**, womit
du den eigenen Wert wieder aufgibst und der Zustand erneut von oben kommt. Hat eine
übergeordnete Organisation festgelegt, dass ihre Untereinheiten keine eigenen Werte
setzen dürfen, steht der Schalter fest, und ein Hinweis nennt die sperrende
Organisation.

Zum Ändern brauchst du zusätzlich zu `setting:read` die Berechtigung `setting:update`;
ohne sie siehst du die Schalter nur. Gespeichert wird wie bei den übrigen Werten dieser
Seite mit dem einen Knopf **Speichern**, der Modulschalter und Fachliche Vorgaben
gemeinsam übernimmt.

![Der Abschnitt Module mit dem Schalter für Bewässerung und seinem Herkunftsabzeichen](../images/settings-organization-modules.png)

Schaltest du ein Modul ab, verschwinden seine Bereiche für alle Mitarbeitenden der
Organisation und ihrer Unterorganisationen aus der Navigation, von der Karte, vom
Dashboard und aus der Einsatzplanung. Wer einen Link auf eine solche Seite öffnet,
sieht stattdessen den Hinweis **Modul nicht aktiv**. Die Daten selbst bleiben unberührt:
Bäume, Bewässerungsgruppen und Einsatzpläne werden weder gelöscht noch verändert, und
sie stehen wieder zur Verfügung, sobald das Modul erneut eingeschaltet wird. Auch die
Rechte, die Rollen für das Modul enthalten, bleiben gespeichert, wirken aber nur,
solange das Modul aktiv ist, siehe
[Team und Rollen](./settings-team.md#der-berechtigungskatalog).

Auch diese Hilfe richtet sich danach: Wer angemeldet ist, sieht darin nur die Kapitel
zu Modulen, die für ihn aktiv sind. Ein direkt geöffneter Link auf ein ausgeblendetes
Kapitel funktioniert trotzdem.

## Fachliche Vorgaben

Mit der Berechtigung `setting:read` zeigt die Detailansicht einer Organisation
zusätzlich den Abschnitt **Fachliche Vorgaben** mit dem Wasserbedarf je Baum in Litern,
der Nachwirkzeit frisch gegossen in Stunden und dem Kartenausschnitt. Der Wasserbedarf geht
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

### Kartenausschnitt

Der **Kartenausschnitt** bestimmt, wo die Karte für die Mitarbeitenden dieser
Organisation aufgeht und welchen Bereich sie überhaupt freigibt. Beides gehört
zusammen: Der Ausschnitt ist die Grenze, über die hinaus sich die Karte nicht schwenken
lässt, und der Mittelpunkt ist der Punkt, an dem sie startet. Ohne eigenen Wert erbt
eine Organisation auch hier nach oben, bis zur Vorgabe der Instanz. Eine Instanz, die
mehrere Kommunen verwaltet, zeigt damit nicht mehr allen denselben Ort.

Anders als die beiden Zahlenwerte wird der Kartenausschnitt nicht eingetippt, sondern
auf einer Karte festgelegt. Nach **Eigenen Wert setzen** wird die Vorschau bedienbar:
Du schwenkst und zoomst, bis der sichtbare Bereich genau den Bereich zeigt, den deine
Mitarbeitenden erreichen können sollen, und ziehst die Markierung an die Stelle, an der
die Karte aufgehen soll. **Ausschnitt übernehmen** schreibt beides in den Entwurf;
gespeichert wird wie bei den übrigen Werten erst mit **Speichern**. Liegt die Markierung
am Ende außerhalb des gewählten Bereichs, weist die Anwendung das zurück und sagt es
unter dem Wert, denn ein Mittelpunkt außerhalb der eigenen Grenze wäre ein Ausschnitt,
den niemand sehen könnte.

Unter der Karte stehen zusätzlich die beiden Zoomstufen, zwischen denen sich die Karte
bewegen darf. Kleine Werte zeigen mehr Fläche, große mehr Detail; die Skala reicht von 0
bis 24, wobei der mitgelieferte Bereich von 13 bis 18 ungefähr vom Stadtgebiet bis zum
einzelnen Baum reicht. Eine untere Stufe hält davon ab, aus dem Arbeitsgebiet
herauszuzoomen, eine obere davon, tiefer hineinzugehen, als die Kartendaten hergeben.
Läuft der Bereich verkehrt herum, also die untere Stufe über der oberen, weist die
Anwendung das unter dem Wert zurück.

Nicht jede Organisation arbeitet in einem abgegrenzten Gebiet. Der Schalter **Keine
Einschränkung** oben rechts hebt die Grenze deshalb ganz auf, und zwar vollständig: Weder
der Ausschnitt noch die Zoomstufen gelten dann noch, die Karte lässt sich überallhin
schwenken und so weit zoomen, wie das Kartenmaterial reicht. Bestehen bleibt nur der
Mittelpunkt, denn aufgehen muss sie weiterhin irgendwo. Solange der Schalter an ist,
spielt der sichtbare Bereich keine Rolle mehr, die Zoomfelder verschwinden, und aus dem
Knopf wird entsprechend **Mittelpunkt übernehmen**. Wer die Grenze später wieder
einführt, bekommt einen Vorschlag rund um den Mittelpunkt samt Standard-Zoombereich, den
er wie gewohnt zurechtschieben kann. Der Schalter gehört zum Wert selbst und lässt sich
erst bedienen, wenn diese Organisation den Kartenausschnitt über **Eigenen Wert setzen**
übernommen hat; bei einem geerbten Ausschnitt gibt es hier nichts aufzuheben.

Die Änderung greift, sobald eine Karte neu geöffnet wird. Wer die Karte bereits vor sich
hat und darin verschoben hat, bleibt an seiner Stelle; das ist gewollt, denn eine Karte,
die einem unter der Hand wegspringt, verliert genau den Ausschnitt, den man gerade
angesehen hat. Ein Link auf eine Karte mit Koordinaten führt weiterhin dorthin, wohin er
zeigt.

![Der Kartenausschnitt im Bearbeitungszustand mit verschiebbarer Karte und Markierung](../images/settings-organization-map-view.png)

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
