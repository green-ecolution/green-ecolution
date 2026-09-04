---
slug: vehicles
title: Fahrzeuge
part: planning
summary: Ein Fahrzeug anlegen, Führerscheinklassen und Wasserkapazität für die Einsatzplanung und ein Fahrzeug archivieren.
routes: ['/vehicles', '/vehicles/$vehicleId', '/vehicles/new']
---

## Fahrzeug anlegen

Auf der Seite **Alle Fahrzeuge** legst du ein neues Fahrzeug über
**Neues Fahrzeug erstellen** an. Pflichtangaben sind Kennzeichen, Fahrzeugmodell,
Fahrzeugtyp (Transporter oder Anhänger), Wasserkapazität, der aktuelle Status, die
erforderliche Führerscheinklasse sowie Höhe, Breite, Länge und Gewicht des Fahrzeugs;
eine kurze Beschreibung ist optional. Kennzeichen müssen eindeutig sein, ein bereits
vergebenes Kennzeichen weist die Anwendung zurück. Die Abmessungen braucht sie, um bei
der Routenberechnung nur Strecken vorzuschlagen, die für dieses Fahrzeug tatsächlich
befahrbar sind.

Unter **Aktueller Status** trägst du selbst ein, ob das Fahrzeug Verfügbar, Nicht
Verfügbar, In Betrieb oder Unbekannt ist, beim Anlegen wie beim Bearbeiten; die
Anwendung setzt diesen Wert nicht automatisch, etwa wenn das Fahrzeug gerade in einem
Einsatzplan unterwegs ist. Der Status dient allein deiner eigenen Übersicht: Bei der
Auswahl eines Fahrzeugs für einen Einsatzplan spielt er keine Rolle.

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
