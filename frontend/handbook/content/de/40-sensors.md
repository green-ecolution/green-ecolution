---
slug: sensors
title: Sensorik
part: sensors
summary: Die Sensorenliste und ihre Zustände, das Aktivieren eines Sensors per QR-Code, Messwerte und Signalqualität auf der Detailseite sowie das Deaktivieren und Neuverknüpfen eines Geräts.
routes: ['/sensors', '/sensors/$sensorId', '/sensors/new']
---

## Die Geräteliste lesen

Die Übersicht unter **Sensoren** listet jeden im System hinterlegten Sensor mit seinem
Verbindungszustand, seiner ID, seinem Erstelldatum und dem Zeitpunkt der letzten
Datenübertragung. Ein Sensor hat keinen eigenen Namen; seine ID ist zugleich seine
LoRaWAN-Kennung und dient in der Liste als Bezeichnung. Ein Klick auf einen Eintrag
führt zur Detailseite; ob und mit welchem Baum der Sensor verknüpft ist, siehst du erst
dort. Wer Sensoren aktivieren darf, findet oben die Schaltfläche **Sensor aktivieren**,
die zum Aktivierungsassistenten führt.

Damit ein Sensor überhaupt in dieser Liste erscheint, muss er im System angelegt sein.
Das geschieht nicht über diese Anwendung: Bringst du eine neue Sensoreinheit ins Feld
und ihre ID ist der Anwendung unbekannt, meldet der Aktivierungsassistent das beim Scan
und verweist auf die Verwaltung oder eine zuständige Administratorin. Erst ein bereits
angelegter Sensor lässt sich hier aktivieren.

![Die Geräteliste mit vorbereiteten, online und offline gemeldeten Sensoren nebeneinander](../images/sensor-list.png)

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

## Sensor aktivieren

Der Aktivierungsassistent unter **Sensor aktivieren** führt in drei Schritten durch das
Verknüpfen einer physischen Sensoreinheit mit einem Baum:

1. **QR-Scan.** Halte den QR-Code auf der Sensoreinheit vor die Kamera. Die Anwendung
   gleicht die gescannte ID mit dem System ab. Ist der Sensor unbekannt oder die ID
   ungültig, weist die Anwendung darauf hin und bietet einen erneuten Scan an. Ist der
   Sensor bereits aktiviert, egal ob **Online** oder **Offline**, lässt er sich hier
   nicht ein zweites Mal aktivieren; ein bereits verknüpftes Gerät wird stattdessen über
   seine Detailseite neu zugeordnet (siehe unten).
2. **Baum zuordnen.** Die Anwendung ermittelt deinen aktuellen Standort und schlägt
   Bäume in der Nähe vor, wahlweise auf einer kleinen Karte oder als Liste. Der passende
   Baum lässt sich auch über die Suche nach Baumnummer oder Baumart finden, falls er
   nicht unter den Vorschlägen ist. Gespeichert wird am Sensor ausschließlich diese
   Baumzuordnung, nicht dein GPS-Standort zum Zeitpunkt der Aktivierung; die Position
   des Sensors auf der Karte ergibt sich später allein aus dem Standort des verknüpften
   Baums (mehr dazu im nächsten Abschnitt).
3. **Zuordnung prüfen.** Ein letzter Überblick zeigt Sensor-ID, gewählten Baum und deine
   aktuelle Position zur Kontrolle. Ein Klick auf **Sensor aktivieren** speichert die
   Verknüpfung; danach lässt sich direkt der nächste Sensor scannen oder zur Übersicht
   zurückkehren.

![Der Aktivierungsassistent im ersten Schritt: QR-Scan der Sensoreinheit](../images/sensor-wizard-qr.png)

## Ein Sensor hat keine eigene Position

Für einen Sensor gibt es kein Feld, in das du Koordinaten einträgst. Sein Standort auf
der Karte der Detailseite ist immer der Standort des Baums, mit dem er gerade verknüpft
ist; ist er keinem Baum zugeordnet, zeigt die Anwendung dort keine Karte, sondern einen
Hinweis, dass der Sensor noch nicht im Feld ist. Wird die Verknüpfung gelöst oder der
Sensor einem anderen Baum zugewiesen, wandert die angezeigte Position entsprechend mit,
ohne dass jemand einen Standort pflegt. Das GPS-Signal, das der Aktivierungsassistent im
Schritt **Baum zuordnen** nutzt, dient nur dazu, dir passende Bäume in deiner Nähe
vorzuschlagen; es wird nicht als Position des Sensors übernommen.

## Messwerte und Signalqualität auf der Detailseite

Der Kopf der Detailseite zeigt Sensortyp, ID, Verbindungszustand, Modell und, falls
vorhanden, den anbindenden Provider. Darunter fassen drei Kennzahlen den aktuellen
Zustand zusammen: **Status**, **Akkustand** und **Letztes Signal** mit dem Zeitpunkt der
letzten Übertragung; ab einer Spannung von 2,8 V schaltet sich die Batterie ab, was die
Anwendung an dieser Stelle vermerkt.

Der Abschnitt **Signal** zeigt die Empfangsqualität der letzten Übertragung als
RSSI-Wert in dBm, eingeordnet in **Gut**, **Ausreichend** oder **Schwach**, dazu SNR und
die Zahl der empfangenden Gateways sowie einen Verlauf der letzten Werte. Darunter folgt
bei Sensoren mit Bodenfeuchte- oder Bodenspannungsmessung ein Verlauf der Messwerte je
Tiefe, mit eingezeichneter kritischer Schwelle und den Zeitpunkten vergangener
Bewässerungen. Wofür diese Messwerte am zugehörigen Baum stehen und wie daraus ein
Bewässerungszustand wird, erklärt das
[Kapitel zu Bewässerungsgruppen](./treecluster.md#bewasserungsstatus-und-wie-er-zustande-kommt).

Erkennt die Anwendung in den letzten sieben Tagen wiederholt unplausible Werte, weist
ein Hinweis zur **Datenqualität** darauf hin; das betrifft die Verlässlichkeit der
Messwerte, nicht den Verbindungszustand, der weiterhin allein aus der Übertragungszeit
folgt. Die Stammdaten darunter fassen Modell, Provider sowie bei LoRaWAN-Sensoren
Seriennummer und Geräteschlüssel zusammen.

![Die Detailseite eines Sensors mit Messwerten und Signalqualität](../images/sensor-detail.png)

## Sensor deaktivieren und neu verknüpfen

Über das Aktionsmenü auf der Detailseite eines aktivierten Sensors stehen **Anderen Baum
zuweisen** und **Baumverknüpfung aufheben** zur Verfügung. **Anderen Baum zuweisen**
löst die bestehende Verknüpfung und legt sie auf einen neu gewählten Baum um, in einem
Schritt; die vorherige Zuordnung am alten Baum entfällt dabei automatisch.

**Baumverknüpfung aufheben** setzt den Sensor vollständig auf **Vorbereitet** zurück und
entfernt die Baumzuordnung. Ein so zurückgesetzter Sensor lässt sich anschließend wie ein
neuer über den Aktivierungsassistenten an einem beliebigen Baum erneut aktivieren, ohne
dass jemand ihn im System neu anlegen muss. Das ist der richtige Weg, wenn eine
Sensoreinheit abgebaut und später an anderer Stelle wieder eingesetzt wird. Davon zu
unterscheiden ist **Sensor löschen**: Diese Aktion entfernt den Sensor endgültig aus dem
System, zusammen mit seiner LoRaWAN-Konfiguration und allen aufgezeichneten Messdaten,
und lässt sich nicht rückgängig machen; die Anwendung fragt vor dem Löschen deshalb
noch einmal nach.
