---
slug: sensor-readings
title: Messwerte und Verknüpfungen
part: sensors
summary: Die Detailseite eines Sensors lesen, von den Kennzahlen über Signalqualität und Messwerte bis zur Datenqualität, und die Verknüpfung mit einem Baum ändern oder wieder lösen.
routes: ['/sensors/$sensorId']
---

Auf der Detailseite wird aus einem Eintrag in der Liste ein Messpunkt am Baum: Sie zeigt,
wie gut das Gerät empfangen wird, was es zuletzt gemessen hat und an welchem Baum es
hängt. Dorthin gelangst du über einen Klick auf einen Eintrag in der
[Geräteliste](./sensors.md#die-gerateliste-lesen).

## Die Detailseite lesen

Der Kopf der Detailseite zeigt Sensortyp, ID, Verbindungszustand, Modell und, falls
vorhanden, den anbindenden Provider. Darunter fassen drei Kennzahlen den aktuellen
Zustand zusammen: **Status**, **Akkustand** und **Letztes Signal** mit dem Zeitpunkt der
letzten Übertragung; ab einer Spannung von 2,8 V schaltet sich die Batterie ab, was die
Anwendung an dieser Stelle vermerkt.

## Signal, Messwerte und Datenqualität

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
folgt. Unter dem Hinweis stehen zunächst die fünf jüngsten auffälligen Messwerte; über
**Alle … anzeigen** klappt die vollständige Liste auf. Die Stammdaten darunter fassen Modell, Provider sowie bei LoRaWAN-Sensoren
Seriennummer und Geräteschlüssel zusammen.

![Die Detailseite eines Sensors mit Messwerten und Signalqualität](../images/sensor-detail.png)

## Sensor deaktivieren und neu verknüpfen

Über das Aktionsmenü auf der Detailseite eines aktivierten Sensors stehen **Anderen Baum
zuweisen** und **Baumverknüpfung aufheben** zur Verfügung. **Anderen Baum zuweisen**
löst die bestehende Verknüpfung und legt sie auf einen neu gewählten Baum um, in einem
Schritt; die vorherige Zuordnung am alten Baum entfällt dabei automatisch.

**Baumverknüpfung aufheben** setzt den Sensor vollständig auf **Vorbereitet** zurück und
entfernt die Baumzuordnung. Ein so zurückgesetzter Sensor lässt sich anschließend wie
ein neuer über den [Aktivierungsassistenten](./sensor-installation.md#sensor-aktivieren)
an einem beliebigen Baum erneut aktivieren, ohne dass jemand ihn im System neu anlegen
muss. Das ist der richtige Weg, wenn eine Sensoreinheit abgebaut und später an anderer
Stelle wieder eingesetzt wird. Davon zu unterscheiden ist **Sensor löschen**: Diese
Aktion entfernt den Sensor endgültig aus dem System, zusammen mit seiner
LoRaWAN-Konfiguration und allen aufgezeichneten Messdaten, und lässt sich nicht
rückgängig machen; die Anwendung fragt vor dem Löschen deshalb noch einmal nach.
