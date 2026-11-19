---
slug: settings-organization
title: Überblick
part: administration
summary: Die Stammdaten einer Organisation, ihr Standort, die Kontaktperson, der Kartenausschnitt und die Verweise zu Mitarbeitenden, Struktur und Aufgabenbereichen.
routes: ['/settings/organization', '/settings/organization/$orgId']
---

> [!NOTE]
> Diese Seite setzt die Berechtigung `organization:read` voraus. Fehlt sie dir, taucht
> **Überblick** in deiner Einstellungsnavigation gar nicht erst auf; wende dich an
> die Verwaltung deiner Organisation, wenn du sie brauchst.

## Die Stammdaten einer Organisation

Eine Organisation ist in Green Ecolution der Besitzer von Personen, Bäumen,
Bewässerungsgruppen, Sensoren, Fahrzeugen und Einsatzplänen und zugleich der Rahmen für
Rollen und Einstellungen. Wie die Organisationen untereinander zusammenhängen, zeigt
die [Struktur](./settings-structure.md). Der **Überblick** ist die Seite einer einzelnen
Organisation. Er öffnet sich für deine eigene Organisation oder für die, die du im
Benutzermenü ausgewählt hast. Welche Organisation die Einstellungen gerade betreffen,
wechselst du über die Organisationswahl in der Navigation der Einstellungen.

Die Seite zeigt zuerst den Namen der Organisation und darunter die Karten
**Stammdaten** mit dem Namen, **Standort** mit Straße, Postleitzahl und Ort sowie
**Kontaktperson**. Die Adresse ist optional, lässt sich aber nur vollständig
speichern. Als Kontaktperson kommen nur Personen infrage, die der Organisation
zugeordnet sind; über **Kontaktperson festlegen** oder **Ändern** wählst du sie aus.
Geändert wird mit der Berechtigung `organization:update` und wirkt erst mit
**Speichern**. Die oberste Organisation der Instanz trägt immer den Namen Green
Ecolution und lässt sich nicht bearbeiten; ein Hinweis auf der Seite sagt das.

![Der Überblick einer Organisation mit der Navigation der Einstellungen in zwei Gruppen](../images/settings-organization.png)

## Kartenausschnitt

Mit der Berechtigung `setting:read` zeigt der Überblick zusätzlich den
**Kartenausschnitt**. Er bestimmt, wo die Karte für die Mitarbeitenden dieser
Organisation aufgeht und welchen Bereich sie überhaupt freigibt. Beides gehört
zusammen: Der Ausschnitt ist die Grenze, über die hinaus sich die Karte nicht schwenken
lässt, und der Mittelpunkt ist der Punkt, an dem sie startet. Ohne eigenen Wert erbt
eine Organisation auch hier nach oben, bis zur Vorgabe der Instanz. Eine Instanz, die
mehrere Kommunen verwaltet, zeigt damit nicht mehr allen denselben Ort.

Anders als die Zahlenwerte unter [Aufgabenbereiche](./settings-areas.md#fachliche-vorgaben) wird der Kartenausschnitt nicht eingetippt, sondern
auf einer Karte festgelegt. Nach **Eigenen Wert setzen** wird die Vorschau bedienbar:
Du schwenkst und zoomst, bis der sichtbare Bereich genau den Bereich zeigt, den deine
Mitarbeitenden erreichen können sollen, und ziehst die Markierung an die Stelle, an der
die Karte aufgehen soll. **Ausschnitt übernehmen** schreibt beides in den Entwurf;
gespeichert wird zusammen mit den Stammdaten erst mit **Speichern**. Liegt die Markierung
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

Wie die Zahlenwerte erbt auch der Kartenausschnitt nach oben und lässt sich von einer
übergeordneten Organisation sperren; ob die Untereinheiten eigene Werte setzen dürfen,
legst du unter [Aufgabenbereiche](./settings-areas.md#fachliche-vorgaben) fest. Zum
Ändern brauchst du zusätzlich `setting:update`.

## Verweise zu den übrigen Seiten

Unter dem Kartenausschnitt stehen bis zu drei Karten, die zu den übrigen Seiten der
Organisation führen und jeweils eine kurze Zusammenfassung zeigen. **Mitarbeitende**
nennt die Zahl der Personen und erscheint mit der Berechtigung `user:read`.
**Struktur** nennt die Zahl der direkten Unterorganisationen oder, dass es noch keine
gibt. **Aufgabenbereiche** erscheint mit `setting:read` und zählt die eingeschalteten
Module auf. Mehr dazu steht unter [Mitarbeitende und Rollen](./settings-team.md),
[Struktur](./settings-structure.md) und [Aufgabenbereiche](./settings-areas.md).

## Eine Organisation löschen

Mit der Berechtigung `organization:delete` steht am Ende der Seite **Organisation
löschen**. Das ist nur möglich, solange die Organisation weder Unterorganisationen noch
zugeordnete Mitarbeitende oder andere Ressourcen wie Bäume oder Sensoren mehr besitzt,
ein Hinweis neben dem Knopf erinnert daran. Die oberste Organisation lässt sich nicht
löschen.
