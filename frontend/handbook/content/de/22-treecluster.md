---
slug: treecluster
title: Bewässerungsgruppen
part: greenspaces
summary: Eine Gruppe anlegen, ihr Bäume zuordnen, das Gruppendashboard lesen und verstehen, wie der Bewässerungsstatus zustande kommt.
routes: ['/treecluster', '/treecluster/$treeclusterId']
---

## Gruppe anlegen

Eine Bewässerungsgruppe fasst mehrere Bäume zusammen, die aufgrund ihrer Nähe und
Standortbedingungen gemeinsam bewässert werden; die Ausstattung einzelner Bäume mit
Sensoren erlaubt dann eine Gesamtaussage über den Zustand der ganzen Gruppe. Angelegt
wird eine Gruppe über die Schaltfläche **Gruppe anlegen**, sowohl in der Auflistung
aller Bewässerungsgruppen als auch [auf der Karte](./map.md#baume-und-gruppen-direkt-auf-der-karte-erfassen),
denn zu einer Gruppe gehören von Anfang an ihre Bäume, die du dort anklickst. Neben Name
und Adresse trägst du die Bodenbeschaffenheit ein und kannst eine kurze Beschreibung
ergänzen; die Region der Gruppe bestimmt die Anwendung anschließend selbst anhand der
Lage ihrer Bäume.

## Bäume zuordnen

Bäume ordnest du einer Gruppe auf zwei Wegen zu: beim Anlegen oder Bearbeiten der Gruppe
[auf der Karte](./map.md), indem du sie dort einzeln anklickst, oder umgekehrt beim
Bearbeiten eines einzelnen Baums über dessen Feld **Bewässerungsgruppe**. Ein Baum kann
zu jedem Zeitpunkt nur einer Gruppe angehören oder gar keiner.

> [!NOTE]
> Eine Bewässerungsgruppe darf nur Bäume ihrer eigenen Organisation enthalten. Klickst
> du beim Anlegen einer Gruppe auf der Karte einen Baum einer anderen Organisation an,
> meldet dir die Anwendung, dass sich dieser Baum in deiner Organisation nicht auswählen
> lässt. Darfst du auch in der anderen Organisation Gruppen anlegen, bietet sie dir
> stattdessen an, die Gruppe gleich für diese anzulegen; bereits ausgewählte Bäume gehen
> dabei verloren, weil eine Gruppe nicht zwei Organisationen gleichzeitig gehören kann.

## Das Gruppendashboard lesen

Der Kopf des Dashboards zeigt den Namen der Gruppe, ihren Bewässerungszustand als Badge,
Adresse, Region und Baumzahl sowie eine hinterlegte Beschreibung. Enthält die Gruppe
noch keine Bäume, weist ein Hinweis darauf hin, dass ihr dadurch auch der Standort
fehlt. Darunter folgen vier Kennzahlen: **Bewässerungszustand (ø)**, **Bodentemperatur**,
**Letzte Messung** und **Letzte Bewässerung**; Letztere aktualisiert sich erst, sobald
ein Einsatzplan mit dieser Gruppe als **Beendet** markiert wird.

Der Bereich **Wasserversorgung** zeichnet das pflanzenverfügbare Wasser über die Zeit,
mit gestrichelten Linien für die Schwellen **mäßig** und **kritisch**; ohne Bäume mit
Sensor oder ohne bekannte Bodenart lässt sich der Verlauf nicht berechnen, und die
Anwendung sagt das an derselben Stelle. Die **Bewässerungshistorie** darunter listet die
abgeschlossenen Einsatzpläne dieser Gruppe auf. In der rechten Spalte folgen der
**Standort** mit einer kleinen Karte und einem Link dorthin, die **Sensorik** mit
Signalstärke, Batteriestand und letzter Übertragung je ausgestattetem Baum, die Liste
der **Bäume** dieser Gruppe und ihre **Stammdaten** mit Region, vorkommenden Baumarten,
Bodenart und Beschreibung. Wer die nötige Berechtigung hat, findet oben zudem
**Gruppe bearbeiten** und, über das Menü daneben, **Gruppe löschen**; gelöscht wird
dabei nur die Gruppe, ihre Bäume bleiben erhalten.

![Das Gruppendashboard mit dem Verlauf der Wasserversorgung](../images/treecluster-dashboard.png)

## Bewässerungsstatus und wie er zustande kommt

Der Bewässerungszustand eines Baums wird ausschließlich aus den Messwerten seines
Sensors berechnet, zusammen mit seinem Pflanzjahr und, je nach Sensortyp, der Bodenart
seiner Gruppe. Ohne Sensor lässt sich für einen Baum nie **In Ordnung**, **Leicht
trocken** oder **Sehr trocken** ermitteln, und er steht ansonsten auf **Unbekannt** —
nur wenn ein Einsatzplan seiner Gruppe abgeschlossen wird, zeigt auch er für eine Weile
**Soeben bewässert**.

Der Bewässerungszustand einer Gruppe ist ein Mehrheitswert über die Zustände ihrer
Bäume mit Sensor. Bäume ohne Sensor zählen dabei nicht mit. Stehen sich zwei Zustände
mit gleich vielen Stimmen gegenüber, entscheidet die Anwendung zugunsten des
alarmierenderen: **Sehr trocken** vor **Leicht trocken** vor **Soeben bewässert** vor
**In Ordnung**. Hat keiner der Bäume der Gruppe einen Sensor oder enthält die Gruppe
noch keine Bäume, ist auch ihr Zustand **Unbekannt**. Der Zustand **Soeben bewässert**
wird gesetzt, sobald ein Einsatzplan für die Gruppe abgeschlossen wird, und fällt nach
einer gewissen Zeit auf **Unbekannt** zurück; einen aus Sensordaten berechneten Zustand
zeigt die Gruppe erst wieder, sobald tatsächlich neue Messwerte eintreffen.

Ist der Zustand einer Gruppe unbekannt, erklärt ein Hinweis auf dem Dashboard den oder
die Gründe dafür: dass keiner ihrer Bäume einen Sensor trägt, dass die Bodenart der
Gruppe fehlt und deshalb Feuchtemesswerte nicht ausgewertet werden können, dass ein
oder mehrere Sensoren derzeit keine Daten senden, dass ein Baum das überwachte
Anwuchsfenster bereits verlassen hat oder dass für einen Baum schlicht keine
auswertbaren Messwerte vorliegen.

> [!IMPORTANT]
> Ein Baum ohne Sensor bleibt für die Bewässerungsplanung unsichtbar: Weder sein
> eigener Zustand noch der seiner Gruppe zeigt, ob er tatsächlich Wasser braucht. Für
> eine verlässliche Einschätzung des Bewässerungsbedarfs sollte jede Gruppe mindestens
> einen Baum mit Sensor enthalten.

Welche Farben und Bezeichnungen zu welchem Zustand gehören, zeigt die
[Legende auf der Karte](./map.md#ebenen-und-legende).

## Bodenbeschaffenheit

Die Bodenbeschaffenheit einer Gruppe wird als Bodenart nach dem KA5-Bodenartendiagramm
angegeben und ist Voraussetzung dafür, dass Sensoren, die die volumetrische
Bodenfeuchte messen, überhaupt einen Bewässerungszustand liefern können. Kennst du die
genaue Bodenart nicht, ermittelst du sie über die Schaltfläche neben dem Auswahlfeld:
Der Dialog **Bodenart bestimmen** lässt dich die Korngrößenanteile Sand, Schluff und Ton
aus einer Bodenprobe eintragen und zeigt anhand des Bodenartendiagramms, welche Bodenart
sich daraus ergibt; mit **Übernehmen** landet sie im Formular.

## Kommentare

Am Fuß des Dashboards können alle, die auf die Gruppe zugreifen dürfen, Kommentare
hinterlassen, etwa um Beobachtungen vor Ort oder Absprachen zur nächsten Bewässerung
festzuhalten. Eigene Kommentare lassen sich nachträglich bearbeiten, was als
**(bearbeitet)** vermerkt wird, oder wieder löschen, wobei die Anwendung vor dem
endgültigen Löschen noch einmal nachfragt; Kommentare anderer lassen sich nur lesen.
