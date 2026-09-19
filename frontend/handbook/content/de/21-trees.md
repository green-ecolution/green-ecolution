---
slug: trees
title: Bäume
part: greenspaces
summary: Die Baumliste mit Suche, Filtern und Sortierung, die Detailseite eines Baums und die Besonderheit von Bäumen aus dem Baumkataster.
routes: ['/trees', '/trees/$treeId']
---

## Die Baumliste und ihre Filter

Die Seite **Bäume** zeigt jeden Baum mit seiner Baumnummer und seiner Art. Ein farbiger
Punkt davor steht für den Bewässerungszustand; verweilst du mit dem Zeiger darauf, nennt
die Anwendung ihn beim Namen. Darunter stehen die Bewässerungsgruppe und der verknüpfte
Sensor, und fehlt eines von beidem, sagt die Zeile das ausdrücklich, statt es einfach
wegzulassen. Ist das Fenster breit genug, ergänzt dieselbe Zeile das Pflanzjahr und den
Zeitpunkt der letzten Bewässerung; auf schmalen Bildschirmen entfallen beide wieder,
damit die Zeile nicht umbricht. Ein Klick auf einen Eintrag führt zur Detailseite. Über
das Suchfeld durchsuchst du die Liste nach Baumnummer und Art, ohne Rücksicht auf Groß-
und Kleinschreibung; die Suche greift erst kurz nach dem letzten Tastendruck, und der
gefundene Teil des Textes erscheint in den Ergebnissen hervorgehoben.

Unter dem Suchfeld stehen vier Filter, die jeweils sofort wirken, ohne dass eine
Bestätigung nötig wäre: nach dem Bewässerungszustand, nach der Bewässerungsgruppe, wo
sich einzelne Gruppen ebenso wählen lassen wie **Mit Gruppe** und **Ohne Gruppe** für die
bloße Zugehörigkeit, unabhängig davon, um welche Gruppe es sich handelt, nach dem
Vorhandensein eines Sensors mit den Optionen **Mit Sensor** und **Ohne Sensor**, sowie
nach dem Pflanzjahr. Wird die Auswahl eines Filters lang, wie bei der
Bewässerungsgruppe und beim Pflanzjahr, bringt er ein eigenes Suchfeld mit, über das du
die Liste einengst, statt sie durchzublättern. Jede gesetzte Auswahl erscheint darunter
als entfernbarer Chip, daneben ein Link, der alle Filter auf einmal zurücksetzt, sowie
die Anzahl der gefundenen Bäume.

Über das Sortiermenü ordnest du die Liste nach Baumnummer, Art, Bewässerungszustand,
Pflanzjahr, letzter Bewässerung oder Bewässerungsgruppe, jeweils auf- oder absteigend;
eine Sortierung nach Bewässerungszustand richtet sich nach der Dringlichkeit, sodass der
schlechteste Zustand zuerst erscheint. Bei der Baumnummer werden die Ziffern als Zahl
verglichen, 9 steht also vor 1005; Nummern mit einem Buchstaben davor bilden dahinter
eigene Blöcke, in denen wieder numerisch sortiert wird. Suchbegriff, Filter, Sortierung
und die aktuelle Seite stehen in der Adresszeile: Eine gefilterte Ansicht lässt sich
damit als Link weitergeben, und der Zurück-Button des Browsers macht eine Filteränderung
rückgängig.

Findet sich zu den aktuellen Filtern kein Baum, bietet die Anwendung an, sie
zurückzusetzen; ist überhaupt noch kein Baum erfasst, weist sie stattdessen darauf hin.
Wer einen neuen Baum anlegen darf, findet oben die Schaltfläche **Neuen Baum erstellen**;
sie führt zur Erfassung [auf der Karte](./map.md#baume-und-gruppen-direkt-auf-der-karte-erfassen),
denn ein Baum braucht immer einen Standort. Dieselben Bäume lassen sich auch direkt
[auf der Karte](./map.md) ansehen.

![Die Baumliste mit Suchfeld, Filterleiste und aktiven Filtern](../images/tree-list.png)

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

![Die Detailseite eines Baums mit Bewässerungszustand und verknüpftem Sensor](../images/tree-detail.png)

## Geplante Bäume

Ein Pflanzjahr darf in der Zukunft liegen. Damit lässt sich eine Pflanzung erfassen,
bevor der Baum tatsächlich im Boden steht. Solange dieses Jahr nicht erreicht ist,
kennzeichnen die Stammdaten den Baum als **Noch nicht gepflanzt** und erklären darunter,
was daraus folgt.

Ein solcher Baum bleibt bewusst ohne Bewässerungszustand. Es gibt noch keine Wurzeln, um
die herum sich Bodenfeuchte messen ließe, deshalb steht er auf **Unbekannt**, und zwar
unabhängig davon, welche Messwerte eintreffen. Aus demselben Grund lässt sich ihm kein
Sensor zuordnen: Das Feld für den Sensor ist im Formular gesperrt, solange das Pflanzjahr
in der Zukunft liegt, und ein Speicherversuch über einen anderen Weg wird abgewiesen.

Sobald das Pflanzjahr erreicht ist, verschwindet die Kennzeichnung von selbst, und der
Baum verhält sich wie jeder andere. Willst du ihm schon vorher einen Sensor geben, korrigiere
zuerst das Pflanzjahr.

> [!NOTE]
> Auch der umgekehrte Weg ist gesperrt: Trägt ein Baum bereits einen Sensor, lässt sich
> sein Pflanzjahr nicht nachträglich in die Zukunft setzen. Entferne in diesem Fall im
> selben Bearbeitungsschritt den Sensor, dann nimmt die Anwendung die Änderung an.

## Baum erfassen und bearbeiten

Sowohl das Anlegen als auch das Bearbeiten eines Baums laufen über die Karte, weil ein
Standort dazu gehört: den Ablauf beschreibt das
[Kartenkapitel](./map.md#baume-und-gruppen-direkt-auf-der-karte-erfassen). Von der
Detailseite eines Baums aus gelangst du über **Baum bearbeiten** direkt dorthin, mit
bereits ausgefüllten Werten und dem Marker an der bisherigen Position; wie beim Anlegen
lässt er sich weiterhin verschieben, um den Standort anzupassen. Über **Baum löschen**
entfernst du den Baum endgültig; die Anwendung fragt vor dem Löschen noch einmal nach,
weil sich die Aktion nicht rückgängig machen lässt.

![Das Formular zum Bearbeiten eines Baums mit Baumnummer, Baumart, Pflanzjahr und Bewässerungsgruppe](../images/tree-form.png)

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
