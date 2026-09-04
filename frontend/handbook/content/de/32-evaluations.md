---
slug: evaluations
title: Auswertung
part: planning
summary: Welche Zahlen die Auswertung zeigt, worauf sie sich beziehen und welche Schlüsse sich daraus ziehen lassen.
routes: ['/evaluations']
---

## Welche Zahlen die Auswertung zeigt

Die Seite **Auswertung aller Daten** fasst den aktuellen Bestand in zwei Blöcken
zusammen. Der erste zeigt die **Anzahl an Bewässerungsgruppen**, die **Anzahl an
Bäumen**, einschließlich manuell erfasster und aus anderen Systemen importierter Bäume,
sowie die **Anzahl an Sensoren**, auch solcher, die aktuell keinem Baum zugeordnet sind.
Der zweite zeigt die **Anzahl an Einsatzplänen**, den **Wasserverbrauch** in Litern sowie
die **Anzahl Mitarbeitende für Einsatzpläne**; Mehrfachzählungen ein und derselben
Person über mehrere Einsatzpläne hinweg rechnet die Anwendung dabei nicht heraus.
Darunter listet sie die Anzahl an Einsatzplänen je Stadtteil sowie die Nutzung der
Fahrzeuge, jeweils mit der Zahl der Einsatzpläne, denen ein Fahrzeug zugeordnet war.

![Die Seite Auswertung aller Daten mit ihren Kennzahlen und Diagrammen](../images/evaluations.png)

## Worauf sich die Zahlen beziehen

Alle Zahlen dieser Seite beziehen sich, wie überall in der Anwendung, nur auf das, was
deine Organisation und deine Rollen dir zeigen, siehe
[Rollen und Sichtbarkeit](./introduction.md#rollen-und-sichtbarkeit). Die Anzahl an
Einsatzplänen sowie die Auflistungen je Stadtteil und je Fahrzeug zählen dabei jeden
Einsatzplan mit, unabhängig von seinem Zustand: Ein noch geplanter oder ein
abgebrochener Einsatz zählt ebenso mit wie ein beendeter. Der Wasserverbrauch dagegen
stammt ausschließlich aus abgeschlossenen Einsatzplänen, denn erst beim Beenden eines
Einsatzes wird die je Bewässerungsgruppe verbrauchte Wassermenge erfasst.

## Welche Schlüsse sich daraus ziehen lassen

Die Zahlen je Stadtteil und je Fahrzeug zeigen, wie oft ein Gebiet oder ein Fahrzeug in
einem Einsatzplan vorkam, nicht, wie oft tatsächlich bewässert wurde: Ein Fahrzeug mit
vielen Einsatzplänen kann darunter auch abgebrochene oder nicht angetretene Fahrten
haben. Für eine Aussage über tatsächlich geleistete Bewässerung ist der Wasserverbrauch
die verlässlichere Kennzahl, da er ausschließlich aus beendeten Einsätzen stammt.

Die Auswertung zeigt außerdem stets den gesamten bisher erfassten Bestand; eine
Einschränkung auf einen bestimmten Zeitraum bietet die Seite derzeit nicht. Ein hoher
Wasserverbrauch sagt daher zunächst nur, dass seit Beginn der Nutzung viel bewässert
wurde, nicht, ob das in der jüngeren Vergangenheit oder in einem bestimmten Monat
geschah.
