---
slug: settings-areas
title: Aufgabenbereiche
part: administration
summary: Welche Fachmodule eine Organisation nutzt und welche fachlichen Vorgaben wie Wasserbedarf und Nachwirkzeit für sie und ihre Unterorganisationen gelten.
routes: ['/settings/organization/$orgId/areas']
---

> [!NOTE]
> Diese Seite setzt die Berechtigung `setting:read` voraus. Fehlt sie dir, taucht
> **Aufgabenbereiche** in deiner Einstellungsnavigation gar nicht erst auf. Zum Ändern
> brauchst du zusätzlich `setting:update`; ohne sie siehst du die Werte und Schalter nur.

## Fachliche Vorgaben

Der Abschnitt **Fachliche Vorgaben** zeigt den Wasserbedarf je Baum in Litern und die
Nachwirkzeit frisch gegossen in Stunden. Den Kartenausschnitt der Organisation findest du
im [Überblick](./settings-organization.md#kartenausschnitt). Der Wasserbedarf geht
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
Eine Änderung wirkt erst mit **Speichern**, das die Vorgaben und die Modulschalter
gemeinsam übernimmt.

## Module

Green Ecolution besteht aus einem gemeinsamen Kern und Fachmodulen. Der Kern umfasst
die Organisationen, Rollen, Sensoren, Fahrzeuge und Startpunkte; ein Fachmodul bringt die
Bereiche mit, die zu einer bestimmten Aufgabe gehören. Das Modul **Bewässerung** liefert
Bäume, Bewässerungsgruppen, Einsatzpläne und die Auswertung samt der zugehörigen
Kartenebenen und Kacheln auf dem Dashboard.

Im Abschnitt **Module** dieser Seite steht für jedes Modul, das diese Instanz
kennt, eine Zeile mit einem Schalter. Ein eingeschaltetes Modul steht der ausgewählten
Organisation und allen Organisationen darunter zur Verfügung. Wie bei den
[Fachlichen Vorgaben](./settings-areas.md#fachliche-vorgaben) zeigt ein Abzeichen neben dem Schalter, woher
der Zustand kommt: ein eigener Wert dieser Organisation, ein von einer übergeordneten
Organisation geerbter Wert oder die Vorgabe der Instanz, solange noch keine Organisation
im Baum einen Wert gesetzt hat. Ohne eigenen Wert folgt eine Organisation also ihrer
übergeordneten. Den Schalter bedienst du direkt: Sobald du ihn umlegst, setzt die
Organisation einen eigenen Wert, und neben dem Schalter erscheint **Wieder erben**, womit
du den eigenen Wert wieder aufgibst und der Zustand erneut von oben kommt. Hat eine
übergeordnete Organisation festgelegt, dass ihre Untereinheiten keine eigenen Werte
setzen dürfen, steht der Schalter fest, und ein Hinweis nennt die sperrende
Organisation. Module, die unter einer solchen Sperre aus sind, erscheinen dann gar nicht
mehr in der Liste.

Was eine übergeordnete Organisation für eine Unterorganisation einstellt, ist für diese
verbindlich. Schaltet etwa die oberste Organisation das Modul Abfall für einen Mandanten
ab, taucht Abfall für alle, die nur bei diesem Mandanten Rechte haben, in der Liste gar
nicht mehr auf. Schaltet sie es für den Mandanten fest ein, steht am Schalter
**Festgelegt von** mit dem Namen der obersten Organisation. Der Schalter lässt sich dort
dann weder umlegen noch mit **Wieder erben** zurücksetzen. Ändern kann ihn nur, wer in
der festlegenden Organisation oder darüber `setting:update` hat; dort bleibt er auch
sichtbar, wenn das Modul aus ist. Ein Wert, den der Mandant selbst gesetzt hat, bleibt
dagegen offen: Die übergeordnete Organisation kann ihn jederzeit überschreiben
und macht ihn damit ebenfalls verbindlich.

Zum Ändern brauchst du zusätzlich zu `setting:read` die Berechtigung `setting:update`;
ohne sie siehst du die Schalter nur. Gespeichert wird wie bei den übrigen Werten dieser
Seite mit dem einen Knopf **Speichern**, der Modulschalter und Fachliche Vorgaben
gemeinsam übernimmt.

![Der Abschnitt Module mit den Schaltern für Bewässerung und Abfall und ihren Herkunftsabzeichen](../images/settings-organization-modules.png)

Schaltest du ein Modul ab, verschwinden seine Bereiche für alle Mitarbeitenden der
Organisation und ihrer Unterorganisationen aus der Navigation, von der Karte, vom
Dashboard und aus der Einsatzplanung. Wer einen Link auf eine solche Seite öffnet,
sieht stattdessen den Hinweis **Modul nicht aktiv**. Die Daten selbst bleiben unberührt:
Bäume, Bewässerungsgruppen und Einsatzpläne werden weder gelöscht noch verändert, und
sie stehen wieder zur Verfügung, sobald das Modul erneut eingeschaltet wird. Auch die
Rechte, die Rollen für das Modul enthalten, bleiben gespeichert, wirken aber nur,
solange das Modul aktiv ist, siehe
[Mitarbeitende und Rollen](./settings-team.md#der-berechtigungskatalog).

Auch diese Hilfe richtet sich danach: Wer angemeldet ist, sieht darin nur die Kapitel
zu Modulen, die für ihn aktiv sind. Ein direkt geöffneter Link auf ein ausgeblendetes
Kapitel funktioniert trotzdem.
