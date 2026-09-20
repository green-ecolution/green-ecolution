---
slug: sensors
title: Sensoren im Blick behalten
part: sensors
summary: Die Geräteliste, die Verbindungszustände vorbereitet, online und offline, und warum ein Sensor keine eigene Position hat.
routes: ['/sensors']
---

Zur Sensorik gehören zwei Seiten, die sich klar trennen lassen. Die eine spielt sich in
der Anwendung ab: Geräte im Blick behalten, ihre Messwerte lesen, sie mit Bäumen
verknüpfen und die Verknüpfung wieder lösen. Die andere spielt sich draußen am Baum ab:
eine Sensoreinheit ins Erdreich einbringen, an das Funkmodul anschließen und einmalig
aktivieren. Dieses Kapitel beginnt bei der Übersicht in der Anwendung; die Detailseite
eines einzelnen Geräts behandelt [Messwerte und Verknüpfungen](./sensor-readings.md), den
Einbau vor Ort das Kapitel [Sensor einbauen und aktivieren](./sensor-installation.md).

## Die Geräteliste lesen

Die Übersicht unter **Sensoren** listet jeden im System hinterlegten Sensor mit seinem
Verbindungszustand, seiner ID und seinem Modellnamen. Ein farbiger Punkt vor der ID steht
für den Verbindungszustand; verweilst du mit dem Zeiger darauf, nennt die Anwendung ihn
beim Namen. Darunter stehen, ob der Sensor mit einem Baum verknüpft ist und der Zeitpunkt
seiner letzten Messung, und ist das Fenster breit genug, ergänzt dieselbe Zeile das
Erstelldatum; auf schmalen Bildschirmen entfällt es wieder, damit die Zeile nicht
umbricht. Gehört der verknüpfte Baum zu einer Bewässerungsgruppe, nennt dieselbe Zeile
deren Namen, sodass du die Geräte einer Gruppe in der Liste erkennst, ohne jeden Eintrag
zu öffnen. Ein Sensor hat keinen eigenen Namen; seine ID ist zugleich seine
LoRaWAN-Kennung und dient in der Liste als Bezeichnung. Ein Klick auf einen Eintrag
führt zur Detailseite; um welchen Baum es sich handelt, siehst du erst dort.

Über das Suchfeld durchsuchst du die Liste nach der Sensor-ID, dem Modellnamen und dem
Namen der Bewässerungsgruppe, zu der der verknüpfte Baum gehört, ohne Rücksicht auf Groß-
und Kleinschreibung; der gefundene Teil des Textes erscheint in den Ergebnissen
hervorgehoben. Darunter stehen fünf Filter, die jeweils sofort wirken, ohne
dass eine Bestätigung nötig wäre: nach dem Verbindungszustand, nach dem Modell, nach der
Datenqualität, danach, ob ein Baum verknüpft ist, mit den Optionen **Mit Baum** und
**Ohne Baum**, und schließlich nach der Bewässerungsgruppe. Der Gruppenfilter listet die
angelegten Gruppen zur Auswahl und lässt sich auf mehrere gleichzeitig setzen; er greift
auf die Gruppe des verknüpften Baums zurück, denn ein Sensor gehört nie selbst zu einer
Gruppe. Über das Sortiermenü ordnest du die Liste nach Sensor-ID, letzter Messung
oder Anlagedatum, jeweils auf- oder absteigend. Jede gesetzte Auswahl erscheint darunter
als entfernbarer Chip, daneben ein Link, der alle Filter auf einmal zurücksetzt, sowie
die Anzahl der gefundenen Sensoren.

Wer Sensoren aktivieren darf, findet oben die Schaltfläche **Sensor aktivieren**, die zum
[Aktivierungsassistenten](./sensor-installation.md#sensor-aktivieren) führt.

Damit ein Sensor überhaupt in dieser Liste erscheint, muss er im System angelegt sein.
Das geschieht nicht über diese Anwendung: Bringst du eine neue Sensoreinheit ins Feld
und ihre ID ist der Anwendung unbekannt, meldet der Aktivierungsassistent das beim Scan
und verweist auf die Verwaltung oder eine zuständige Administratorin. Erst ein bereits
angelegter Sensor lässt sich hier aktivieren.

![Die Geräteliste mit Suchfeld, Filterleiste und aktiven Filtern](../images/sensor-list.png)

## Die Verbindungszustände: vorbereitet, online und offline

Ein Sensor meldet nicht selbst, ob er online ist. Sein Verbindungszustand wird bei jedem
Aufruf neu berechnet, aus zwei Angaben, die die Anwendung ohnehin kennt: ob der Sensor
aktiviert wurde und wie lange seine letzte Datenübertragung zurückliegt. Ein Gerät kann
also nicht behaupten, in Ordnung zu sein; ausbleibende Daten schlagen sich zuverlässig im
angezeigten Zustand nieder.

| Zustand         | Bedeutung                                                                                                                                                                                               |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Vorbereitet** | Der Sensor ist im System registriert, aber noch nicht aktiviert und keinem Baum zugeordnet. Dieser Zustand bleibt bestehen, ganz gleich, wie frisch oder alt eine eventuell vorhandene Übertragung ist. |
| **Online**      | Der Sensor ist aktiviert, und seine letzte Übertragung liegt nicht länger als 24 Stunden zurück.                                                                                                        |
| **Offline**     | Der Sensor ist aktiviert, aber seit mehr als 24 Stunden ist keine Übertragung eingegangen, oder es liegt noch nie eine vor.                                                                             |

> [!NOTE]
> Ein als **Offline** angezeigter Sensor ist nicht zwangsläufig defekt. Die Anwendung
> weiß nur, dass die letzten 24 Stunden ohne Datenübertragung vergangen sind; ob das an
> einer leeren Batterie, einem Funkloch oder einer echten Störung liegt, lässt sich von
> hier aus nicht unterscheiden. Ein Blick auf die Detailseite mit dem Zeitpunkt der
> letzten Übertragung und dem Akkustand ist der nächste Schritt, keine Ferndiagnose.

## Ein Sensor hat keine eigene Position

Für einen Sensor gibt es kein Feld, in das du Koordinaten einträgst. Sein Standort auf
der Karte der Detailseite ist immer der Standort des Baums, mit dem er gerade verknüpft
ist; ist er keinem Baum zugeordnet, zeigt die Anwendung dort keine Karte, sondern einen
Hinweis, dass der Sensor noch nicht im Feld ist. Wird die Verknüpfung gelöst oder der
Sensor einem anderen Baum zugewiesen, wandert die angezeigte Position entsprechend mit,
ohne dass jemand einen Standort pflegt. Das GPS-Signal, das der Aktivierungsassistent im
Schritt **Baum zuordnen** nutzt, dient nur dazu, dir passende Bäume in deiner Nähe
vorzuschlagen; es wird nicht als Position des Sensors übernommen.
