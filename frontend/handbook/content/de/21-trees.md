---
slug: trees
title: Bäume
part: greenspaces
summary: Die Baumliste und ihre Filter, die Detailseite eines Baums und die Besonderheit von Bäumen aus dem Baumkataster.
routes: ['/trees', '/trees/$treeId']
---

## Die Baumliste und ihre Filter

Die Auflistung aller Bäume zeigt jeden Baum mit seinem Bewässerungszustand, seiner
Baumart, seiner Baumnummer und der Bewässerungsgruppe, der er zugeordnet ist. Ein Klick
auf einen Eintrag führt zu dessen Detailseite; ohne zugeordnete Gruppe steht dort
**Nicht zugeordnet**. Über den Filter engst du die Liste nach dem Zustand der
Bewässerung, nach der Zugehörigkeit zu einer Gruppe und nach dem Pflanzjahr ein. Wer
einen neuen Baum anlegen darf, findet oben die Schaltfläche **Neuen Baum erstellen**; sie
führt zur Erfassung [auf der Karte](./map.md#baume-und-gruppen-direkt-auf-der-karte-erfassen),
denn ein Baum braucht immer einen Standort. Dieselben Bäume lassen sich auch direkt
[auf der Karte](./map.md) ansehen.

## Die Detailseite lesen

Der Kopf der Detailseite zeigt die Baumnummer, den aktuellen Bewässerungszustand als
Badge sowie Baumart und die zugeordnete Bewässerungsgruppe mit ihrer Adresse. Darunter
folgen der **Bewässerungszustand** und der Zeitpunkt der **Letzten Bewässerung** als
Kennzahlen; Letzteres aktualisiert sich erst, sobald ein Einsatzplan mit diesem Baum als
**Beendet** markiert wird, nicht schon während der Fahrt.

Die Karte im Abschnitt **Standort** zeigt den Baum an seiner Position, mit einem Link
**Auf der Karte anzeigen**, der direkt dorthin führt. Der Abschnitt **Bewässerungsgruppe**
verlinkt auf das Dashboard der zugeordneten Gruppe und zeigt deren Baumzahl und Region;
weicht der Zustand des Baums vom Zustand der Gruppe ab, weist ein Hinweis darauf hin,
dass der Gruppenzustand ein Mehrheitswert ist. Ist der Baum keiner Gruppe zugeordnet,
erklärt die Anwendung an derselben Stelle, dass ohne Gruppe die Bodenart fehlt und sich
der Bewässerungszustand deshalb nicht aus Feuchtemesswerten berechnen lässt.

Der Abschnitt **Sensor** zeigt bei einem ausgestatteten Baum Signalstärke, Batteriestand
und den Zeitpunkt der letzten Übertragung. Ist kein Sensor verbaut, weist die Anwendung
ausdrücklich darauf hin, dass der Bewässerungszustand deshalb unbekannt bleibt. Die
**Stammdaten** darunter fassen Baumart, Baumnummer, Pflanzjahr, den Ursprung der Daten,
die Koordinaten und den Zeitpunkt der letzten Aktualisierung zusammen.

## Baum erfassen und bearbeiten

Sowohl das Anlegen als auch das Bearbeiten eines Baums laufen über die Karte, weil ein
Standort dazu gehört: den Ablauf beschreibt das
[Kartenkapitel](./map.md#baume-und-gruppen-direkt-auf-der-karte-erfassen). Von der
Detailseite eines Baums aus gelangst du über **Baum bearbeiten** direkt dorthin, mit
bereits ausgefüllten Werten und dem Marker an der bisherigen Position; wie beim Anlegen
lässt er sich weiterhin verschieben, um den Standort anzupassen. Über **Baum löschen**
entfernst du den Baum endgültig; die Anwendung fragt vor dem Löschen noch einmal nach,
weil sich die Aktion nicht rückgängig machen lässt.

## Bäume aus dem Baumkataster

Manche Bäume stammen nicht aus einer manuellen Erfassung, sondern wurden aus dem
kommunalen Baumkataster übernommen. Auf der Detailseite erkennst du das am Feld
**Ursprung der Daten** in den Stammdaten: Bei einem manuell erfassten Baum steht dort
**Manuell erstellt**, bei einem übernommenen Baum die Kennung des liefernden Systems.

> [!NOTE]
> Bei einem Baum aus dem Kataster fehlen im Bearbeitungsformular die Felder für
> Baumnummer, Baumart, Pflanzjahr und Bewässerungsgruppe vollständig, der Marker lässt
> sich nicht mehr verschieben und es gibt keine Schaltfläche zum Löschen. Diese Angaben
> pflegt das Katastersystem, nicht Green Ecolution. Bearbeiten lassen sich bei einem
> solchen Baum nur der zugeordnete Sensor und die Kurzbeschreibung.
