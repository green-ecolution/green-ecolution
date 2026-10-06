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

Der Kopf der Detailseite zeigt ein Foto des Sensormodells, die ID des Sensors mit seinem
Verbindungszustand, darunter Modell, Sensortyp und, falls vorhanden, den anbindenden
Provider. Darunter fassen vier
Kennzahlen den aktuellen Zustand zusammen: **Status**, **Akkustand**, **Letztes Signal**
mit dem Zeitpunkt der letzten Übertragung und bei LoRaWAN-Sensoren **Empfang**. Ab einer
Spannung von 2,8 V schaltet sich die Batterie ab; das Infosymbol neben dem Akkustand
erinnert daran.

Auf einem breiten Bildschirm teilt sich die Seite darunter in zwei Spalten. Links stehen
die Verläufe, rechts, wo der Sensor sitzt und was ihn ausmacht. Auf Tablet und Smartphone
folgen die Abschnitte untereinander, der Einsatzort direkt nach dem Messverlauf.

## Einsatzort und Messprofil

Der Abschnitt **Einsatzort** zeigt den Sensor auf einer kleinen Karte zwischen den Bäumen
der Umgebung. Darunter steht die Bewässerungsgruppe mit ihrem Bewässerungszustand und der
Zahl ihrer Bäume, und darunter der Baum, an dem der Sensor steckt, mit Baumnummer, Art und
Pflanzjahr. Beide Zeilen führen per Klick auf die jeweilige Detailseite. Gehört der Baum
zu keiner Bewässerungsgruppe, steht das an dieser Stelle. Ist der Sensor noch mit keinem
Baum verknüpft, erscheint stattdessen ein Hinweis mit der Schaltfläche **Aktivieren &
Baum zuweisen**. Über **Koordinaten** lässt sich die genaue Position einblenden, die
immer vom verknüpften Baum übernommen wird.

Das **Messprofil** zeigt als schmalen Tiefenschnitt, in welchen Tiefen der Sensor misst
und was er dort erfasst, etwa Bodenfeuchtigkeit in Prozent oder Temperatur in °C. Die
Punkte haben dieselbe Farbe wie die Linie der jeweiligen Tiefe im Messverlauf, so lassen
sich Profil und Kurve einander zuordnen.

## Signal, Messwerte und Datenqualität

Bei Sensoren mit Bodenfeuchte- oder Bodenspannungsmessung steht ganz oben ein Verlauf der
Messwerte je Tiefe, mit eingezeichneter kritischer Schwelle und den Zeitpunkten
vergangener Bewässerungen. Wofür diese Messwerte am zugehörigen Baum stehen und wie
daraus ein Bewässerungszustand wird, erklärt das
[Kapitel zu Bewässerungsgruppen](./treecluster.md#bewasserungsstatus-und-wie-er-zustande-kommt).

Die Kennzahl **Empfang** zeigt die Empfangsqualität der letzten Übertragung als RSSI-Wert
in dBm, eingeordnet in **Gut**, **Ausreichend** oder **Schwach**. Der Abschnitt
**Signalverlauf** ergänzt SNR und die Zahl der empfangenden Gateways sowie den Verlauf der
letzten Werte.

Erkennt die Anwendung in den letzten sieben Tagen wiederholt unplausible Werte, erscheint
direkt unter dem Kopf ein Hinweis zur **Datenqualität**; das betrifft die Verlässlichkeit
der Messwerte, nicht den Verbindungszustand, der weiterhin allein aus der
Übertragungszeit folgt. Unter **Betroffene Messwerte** lässt sich die Liste der
verworfenen Werte aufklappen; sie zeigt zunächst die fünf jüngsten, über **Alle …
anzeigen** die vollständige Liste. Die **Stammdaten** fassen bei LoRaWAN-Sensoren Dev EUI,
App EUI, Seriennummer und Geräteschlüssel zusammen, dazu wann der Sensor angelegt und
zuletzt aktualisiert wurde.

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
