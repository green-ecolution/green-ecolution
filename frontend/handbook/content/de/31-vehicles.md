---
slug: vehicles
title: Fahrzeuge
part: planning
summary: Ein Fahrzeug anlegen, Führerscheinklassen und Wasserkapazität für die Einsatzplanung und ein Fahrzeug archivieren.
routes: ['/vehicles', '/vehicles/$vehicleId', '/vehicles/new']
---

## Die Fahrzeugliste lesen

Die Seite **Alle Fahrzeuge** zeigt jedes Fahrzeug mit seinem Kennzeichen und seinem Modell.
Ein farbiger Punkt davor steht für den Status; verweilst du mit dem Zeiger darauf, nennt
die Anwendung ihn beim Namen. Darunter stehen der Fahrzeugtyp und die Wasserkapazität, und
ist das Fenster breit genug, ergänzt dieselbe Zeile die erforderliche Führerscheinklasse;
auf schmalen Bildschirmen entfällt sie wieder, damit die Zeile nicht umbricht. Ein
archiviertes Fahrzeug trägt zusätzlich die Kennzeichnung **Archiviert**. Ein Klick auf
einen Eintrag führt zur Detailseite. Über das Suchfeld durchsuchst du die Liste nach
Kennzeichen, Modell und Beschreibung, ohne Rücksicht auf Groß- und Kleinschreibung; der
gefundene Teil des Textes erscheint in den Ergebnissen hervorgehoben.

Unter dem Suchfeld stehen vier Filter, die jeweils sofort wirken, ohne dass eine
Bestätigung nötig wäre: nach dem Status, nach dem Fahrzeugtyp, nach der
Führerscheinklasse und nach dem Archiv. Der Archiv-Filter ist eine Einzelauswahl: Ein
archiviertes Fahrzeug bleibt so lange aus der Liste ausgeblendet, bis du **Nur
archivierte** wählst, um ausschließlich archivierte Fahrzeuge zu sehen, oder **Inkl.
archivierte**, um sie zusammen mit den aktiven anzuzeigen; ein erneuter Klick auf die
gewählte Option oder das Entfernen ihres Chips bringt dich zurück zur Standardansicht mit
ausschließlich aktiven Fahrzeugen. Über das Sortiermenü ordnest du die Liste nach
Kennzeichen, Wasserkapazität, Modell oder Fahrzeugtyp, jeweils auf- oder absteigend. Jede
gesetzte Auswahl erscheint darunter als entfernbarer Chip, daneben ein Link, der alle
Filter auf einmal zurücksetzt, sowie die Anzahl der gefundenen Fahrzeuge.

![Die Fahrzeugliste mit Suchfeld, Filterleiste und aktiven Filtern](../images/vehicle-list.png)

## Fahrzeug anlegen

Auf der Seite **Alle Fahrzeuge** legst du ein neues Fahrzeug über
**Neues Fahrzeug erstellen** an. Pflichtangaben sind Kennzeichen, Fahrzeugmodell,
Fahrzeugtyp (Transporter oder Anhänger), Wasserkapazität, die Verfügbarkeit, die
erforderliche Führerscheinklasse sowie Höhe, Breite, Länge und Gewicht des Fahrzeugs;
eine kurze Beschreibung ist optional. Kennzeichen müssen eindeutig sein, ein bereits
vergebenes Kennzeichen weist die Anwendung zurück. Die Abmessungen braucht sie, um bei
der Routenberechnung nur Strecken vorzuschlagen, die für dieses Fahrzeug tatsächlich
befahrbar sind.

Unter **Verfügbarkeit** trägst du ein, ob das Fahrzeug Verfügbar oder Nicht Verfügbar
ist, beim Anlegen wie beim Bearbeiten. Gemeint ist damit, ob es grundsätzlich zur
Verfügung steht; Nicht Verfügbar setzt du zum Beispiel während eines
Werkstattaufenthalts. Ob ein Fahrzeug gerade unterwegs ist, trägst du dagegen nicht
selbst ein.

Den Status **Im Einsatz** vergibt die Anwendung selbst: Ein Fahrzeug bekommt ihn, sobald
es als Transporter oder Anhänger an einem Einsatzplan hängt, der gerade läuft, und
verliert ihn wieder, sobald dieser Plan abgeschlossen oder abgebrochen ist. Du musst also
nach einer Fahrt nichts zurücksetzen. Ein Fahrzeug, das du auf Nicht Verfügbar gesetzt
hast, bleibt auch dann Nicht Verfügbar, wenn es noch an einem laufenden Plan hängt: Deine
Angabe hat Vorrang vor einem Plan, den vielleicht nur niemand abgeschlossen hat.

Bei der Auswahl eines Fahrzeugs für einen Einsatzplan spielt weder die Verfügbarkeit noch
der Status eine Rolle; beides dient deiner Übersicht.

![Das Formular zum Anlegen eines Fahrzeugs mit Führerscheinklasse und Wasserkapazität](../images/vehicle-form.png)

## Führerscheinklassen

Green Ecolution kennt die Führerscheinklassen **B**, **BE**, **C** und **CE**. Jedes
Fahrzeug braucht bei seinem Anlegen eine dieser Klassen als Mindestanforderung. Die
Klassen decken sich dabei nicht in einer einfachen Reihenfolge ab: CE deckt B, BE und C
ab; C deckt B ab, aber nicht BE; BE wiederum deckt B ab, aber nicht C.

Bei der Planung eines Einsatzes prüft die Anwendung, ob unter den zugewiesenen
Mitarbeitenden mindestens eine Person alle nötigen Führerscheinklassen besitzt, für
Transporter und einen eventuell verknüpften Anhänger zusammengenommen. Reicht keine der
ausgewählten Personen für die gewählten Fahrzeuge aus, meldet das Formular, dass kein
ausgewählter Mitarbeiter alle erforderlichen Führerscheine für die gewählten Fahrzeuge
hat, und lässt sich nicht absenden.

## Wasserkapazität

Die Wasserkapazität eines Fahrzeugs gibt in Litern an, wie viel Wasser es fasst. Bei der
Auswahl der Bewässerungsgruppen für einen Einsatzplan zählt die Kapazität von Transporter
und, falls einer verknüpft ist, Anhänger zusammen; Gruppen, deren Bäume zusammen mehr
Wasser benötigen, als diese Summe hergibt, lassen sich für den Einsatz nicht auswählen,
siehe [Route festlegen](./watering-plans.md#route-festlegen).

## Fahrzeug archivieren

Ein Fahrzeug, das nicht mehr im Einsatz ist, entfernst du nicht, sondern archivierst es
über **Archivieren** auf seiner Bearbeitungsseite; die Anwendung fragt vor dem
endgültigen Archivieren noch einmal nach. Ein archiviertes Fahrzeug verschwindet aus der
Fahrzeugliste und lässt sich bei neuen Einsatzplänen nicht mehr auswählen. In bereits
bestehenden Einsatzplänen, die es vor der Archivierung als Transporter oder Anhänger
führten, bleibt es weiterhin sichtbar, dort mit dem Zusatz **(Archiviert)** gekennzeichnet.
