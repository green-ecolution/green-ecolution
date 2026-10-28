---
slug: waste-bins
title: Abfallbehälter und Leerungstouren
part: planning
summary: Abfallbehälter erfassen, einen Sensor zuordnen, die Füllstufe lesen, den Schwellwert für voll einstellen sowie Leerungstouren planen und abschließen.
routes:
  ['/waste-bins', '/waste-bins/$wasteBinId', '/waste-bins/new', '/waste-bins/$wasteBinId/edit']
module: waste
---

Das Modul **Abfall** erweitert Green Ecolution um Abfallbehälter und ihre Leerung. Es ist
standardmäßig ausgeschaltet und wird je Organisation im Abschnitt **Module** der
[Organisationsseite](./settings-organization.md#module) eingeschaltet. Erst dann erscheinen
der Bereich **Abfallbehälter** unter **Bestand** in der Seitennavigation, die Behälter auf
der Karte und die Leerungstouren in der Einsatzplanung.

## Die Behälterliste lesen

Unter **Abfallbehälter** stehen alle Behälter, auf die du zugreifen darfst. Über das
Suchfeld findest du einen Behälter anhand seiner Nummer, und der Filter **Füllstufe**
schränkt die Liste auf Behälter in einem bestimmten Zustand ein. Jede Zeile nennt die
Behälternummer, das Volumen, die aktuelle Füllstufe und den zugeordneten Sensor, sofern
es einen gibt. Behälter ohne Sensor tragen den Hinweis **Kein Sensor**.

![Die Liste der Abfallbehälter mit Suche, Füllstufenfilter und den Füllstufen der einzelnen Behälter](../images/waste-bins-list.png)

## Einen Behälter erfassen

Über **Neuen Behälter erfassen** öffnet sich das Formular. Du trägst die Behälternummer
ein, etwa B-0042, das Volumen in Litern und die Tiefe in Zentimetern. Die Tiefe ist der
Abstand vom Sensor bis zum Behälterboden; aus ihr berechnet sich später der Füllstand.
Dazu wählst du die Organisation, der der Behälter gehört, und setzt den Standort mit einem
Klick auf die Karte. Den Marker kannst du danach noch verschieben. Nach dem Speichern
erscheint der Behälter in der Liste. Über **Behälter bearbeiten** auf der Detailseite
änderst du Nummer, Größe und Standort später, und im Menü **Weitere Aktionen** löschst du
einen Behälter nach einer Rückfrage. Dabei wird er aus allen geplanten Leerungstouren
genommen, seine Kommentare werden gelöscht, und ein verbundener Sensor wird freigegeben,
bleibt aber aktiviert.

## Einen Sensor zuordnen

Den Füllstand meldet ein Abstandssensor, der von oben die Entfernung zur Füllung misst.
Als Sensormodell bringt das Modul bislang **GE Distance (simuliert)** mit, einen
simulierten Ultraschallsensor, der den Abstand vom Deckel zur Füllung meldet. Die
Zuordnung läuft über die [Sensorverwaltung](./sensor-installation.md). Beim Aktivieren
eines Sensors wählst du als Ziel einen Abfallbehälter aus, den die Anwendung nach der
Entfernung zu deinem Standort sortiert. Alternativ weist du in der Sensordetailansicht
einen anderen Behälter zu oder hebst die Verknüpfung über **Behälterverknüpfung
aufheben** wieder auf. Ein Behälter kann höchstens einen Sensor haben, und ein Sensor
sitzt an höchstens einem Behälter. Beide müssen zur selben Organisation gehören.

## Die Detailseite

Die Detailseite eines Behälters zeigt links die **Stammdaten** mit Nummer, Volumen,
Tiefe, Koordinaten, Datenquelle und dem Zeitpunkt der letzten Änderung. Rechts steht der
Bereich **Füllstand** mit der Füllstufe, dem Füllstand in Prozent, dem Zeitpunkt der
letzten Messung und der letzten Leerung sowie dem verbundenen Sensor.

Darunter können alle, die auf den Behälter zugreifen dürfen, Kommentare hinterlassen,
etwa zu einem beschädigten Deckel oder einem zugestellten Standplatz. Eigene Kommentare
lassen sich nachträglich bearbeiten, was als **(bearbeitet)** vermerkt wird, oder nach
einer Rückfrage löschen. Kommentare anderer lassen sich nur lesen.

![Die Detailseite eines Abfallbehälters mit Stammdaten und Füllstand](../images/waste-bin-detail.png)

## Wie die Füllstufe zustande kommt

Der Füllstand ergibt sich aus der letzten Abstandsmessung des Sensors. Misst der Sensor
60 cm bei einer Behältertiefe von 120 cm, ist der Behälter zu 50 Prozent gefüllt; allgemein
gilt Füllstand gleich eins minus Abstand durch Tiefe. Daraus leitet Green Ecolution eine
von vier Füllstufen ab. **Voll** heißt, der Füllstand hat den Schwellwert der Organisation
erreicht. **Normal** gilt darunter.

**Unbekannt** erscheint zuerst, wenn es noch keine Messung gibt oder die letzte Messung
älter ist als die Offline-Grenze der Organisation. Das ist dieselbe Grenze, nach der ein
Sensor als offline gilt, siehe
[Die Verbindungszustände](./sensors.md#die-verbindungszustande-vorbereitet-online-und-offline).
Ein Behälter, der nie gemessen hat, etwa weil er keinen Sensor trägt, bleibt deshalb
dauerhaft auf Unbekannt, auch wenn eine Leerungstour für ihn abgeschlossen wird.

**Geleert** steht ein Behälter nur dann, wenn die letzte Messung noch innerhalb der
Offline-Grenze liegt und der Behälter danach durch eine abgeschlossene Leerungstour
geleert wurde, denn die alte Messung beschreibt dann nicht mehr den Behälter. Ist die
Messung bereits zu alt, zeigt der Behälter Unbekannt. Sobald der Sensor wieder misst,
gilt die neue Messung.

## Den Schwellwert für voll einstellen

Ab welchem Füllstand ein Behälter als voll gilt, legt jede Organisation selbst fest. Die
Vorgabe liegt bei 80 Prozent. Den Wert findest du in den Einstellungen der Organisation
unter **Schwellwert für „voll“**. Er wird wie die
[Fachlichen Vorgaben](./settings-organization.md#fachliche-vorgaben) im Organisationsbaum
nach unten vererbt, und eine Unterorganisation kann einen eigenen Wert setzen, sofern die
übergeordnete Organisation das nicht sperrt. Der Schwellwert bestimmt die Füllstufe in
Liste und Karte und welche Behälter die Tourenplanung vorschlägt. Bereits geplante Touren
bleiben bei einer Änderung unverändert.

## Behälter auf der Karte

Wenn das Modul aktiv ist, zeigt die Karte die Behälter nach Füllstufe eingefärbt. Ein
Klick auf einen Behälter öffnet ein Fenster mit der Behälternummer und dem Link **Zum
Behälter**, der zur Detailseite führt. Führt ein gespeicherter Link auf einen Behälter, der
inzwischen gelöscht oder für dich nicht sichtbar ist, meldet das Fenster stattdessen, dass
der Behälter nicht gefunden wurde.

## Leerungstouren planen

Leerungstouren legst du auf demselben Board an wie Bewässerungseinsätze, siehe
[Einsatzpläne](./watering-plans.md). Sind mehrere Planungsmodule aktiv, wechselst du über
den Umschalter zwischen **Bewässerung** und **Leerung**; die Leerung hat die Adresse
`/plans?module=waste`. In der Spalte **Vorschläge** stehen dann die vollen Behälter. Du
markierst einen oder mehrere und klickst auf **Zu Leerungstour bündeln**, oder du wählst
die Behälter im Formular über **Behälter auswählen** aus. Dort wählt **Alle vollen
auswählen** sämtliche Behälter ab dem Schwellwert auf einmal aus.

Wie bei jedem Einsatzplan brauchst du ein Datum, ein Fahrzeug, die Mitarbeitenden und
einen Startpunkt. Für Leerungstouren gelten zwei zusätzliche Voraussetzungen. Das
Fahrzeug muss einen **Laderaum** haben, den du im
[Fahrzeugformular](./vehicles.md) einträgst, und unter **Einstellungen → Stationen** muss
es mindestens eine Station mit der Angabe **Laderaum entladen** geben, siehe
[Stationen](./settings-stations.md). An dieser Entladestelle leert das Fahrzeug seinen
Laderaum, bevor es zurückkehrt. Fehlt eine von beiden, kann Green Ecolution keine Route
berechnen und weist darauf hin.

Der Bedarf eines Behälters ist sein Volumen mal sein Füllstand. Ein Behälter mit der
Füllstufe Geleert zählt mit null Litern. Ist die Füllung unbekannt, rechnet die Planung
vorsichtig mit dem vollen Volumen. Aus der Summe berechnet Green Ecolution, wie viel
Laderaum die Tour braucht.

## Eine Leerungstour abschließen

Beim Beenden einer Tour fragt Green Ecolution für jeden Behälter, ob er geleert wurde.
Vorausgewählt ist **geleert**. Einen Behälter, den die Fahrerin oder der Fahrer nicht
leeren konnte, schaltest du aus. Zusätzlich kannst du für jeden Behälter die gesammelte
Menge in Litern eintragen; die Angabe ist freiwillig. Auf der Detailseite der Tour steht
danach, wie viele Behälter geleert wurden. Deren Füllstufe wechselt auf **Geleert**,
solange ihre letzte Messung noch nicht älter als die Offline-Grenze ist, sonst steht sie
auf Unbekannt, siehe
[Wie die Füllstufe zustande kommt](./waste-bins.md#wie-die-fullstufe-zustande-kommt).

![Der Dialog zum Beenden einer Leerungstour mit der Angabe je Behälter, ob er geleert wurde](../images/waste-plan-outcome.png)

## Rechte und Rolle

Zwei Berechtigungen steuern das Modul. `waste_bin` regelt, wer Behälter ansehen, anlegen,
bearbeiten und löschen darf, `collection_plan` dasselbe für Leerungstouren. Das Anlegen
einer Tour setzt zusätzlich das Recht voraus, Startpunkte anzusehen. Die mitgelieferte
Rollenvorlage **Abfallsammlung** bündelt alles, was für die tägliche Arbeit nötig ist:
Behälter und Touren verwalten sowie Sensoren, Fahrzeuge, Regionen und Startpunkte
ansehen. Die Rechte wirken nur in Organisationen, in denen das Modul aktiv ist, siehe
[Team und Rollen](./settings-team.md#der-berechtigungskatalog).
