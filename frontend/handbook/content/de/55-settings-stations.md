---
slug: settings-stations
title: Stationen
part: administration
summary: Wozu Startpunkte und Stationen dienen, was Fahrzeuge an einer Station tun können, wie sich eine Station anlegen, bearbeiten und löschen lässt und was der Standard-Startpunkt ist.
routes: ['/settings/stations']
---

> [!NOTE]
> Diese Seite siehst du nur mit der Berechtigung `start_point:read`, und nur, wenn deine
> Instanz die Routenberechnung eingeschaltet hat. Fehlt dir die Berechtigung oder ist
> das Routing ausgeschaltet, taucht **Stationen** in deiner Einstellungsnavigation gar
> nicht erst auf. Zum Anlegen, Ändern und Löschen brauchst du zusätzlich
> `start_point:create`, `start_point:update` beziehungsweise `start_point:delete`, jeweils
> in der Organisation, der die Station gehört.

## Wozu Startpunkte und Stationen dienen

Jede Tour eines Einsatzplans beginnt an einem Startpunkt und kehrt dorthin zurück. Manche
dieser Orte sind zugleich Stationen: Dort können Fahrzeuge unterwegs auffüllen oder
entladen. Beides pflegst du auf derselben Seite **Stationen**, denn technisch ist eine
Station ein Startpunkt mit einer Angabe dazu, was dort geschieht.

Diese Angabe steht unter **Fahrzeuge können hier** und kennt zwei Möglichkeiten.
**Wasser auffüllen** bedeutet, dass ein Fahrzeug hier den Tank wieder füllen kann, etwa an
einem Hydranten oder auf dem Betriebshof. **Laderaum entladen** bedeutet, dass ein
Fahrzeug hier seinen Laderaum leeren kann, etwa an einem Wertstoffhof. Eine Station kann beides zugleich anbieten. Wählst du nichts
aus, dient der Ort nur als Start und Ziel einer Tour. Bei der Routenberechnung kommen nur
Stationen infrage, die zur Kapazität des jeweiligen Einsatzplans passen.

![Die Liste der Stationen mit zwei Einträgen, einer davon als Standard markiert](../images/settings-stations.png)

## Der Standard-Startpunkt

Genau ein Startpunkt je Organisation gilt als **Standard**; er ist in der Liste mit dem
Abzeichen **Standard** markiert und wird vorgeschlagen, wenn du einen Einsatzplan
anlegst. Über **Als Standard setzen** machst du einen anderen Startpunkt zum Standard,
das bisherige Abzeichen wandert dann mit. Den Standard selbst kannst du nicht löschen:
Bestimme zuerst einen anderen, dann lässt sich der bisherige entfernen.

Welche Startpunkte und Stationen für einen Einsatzplan infrage kommen, richtet sich nach
der Organisation des Plans: ihre eigenen, dazu die der übergeordneten Organisationen und
die ihrer Untereinheiten. Hat eine Untereinheit keinen eigenen Standard, startet ihre Tour
am Standard der nächsthöheren Organisation. Startpunkte anderer Organisationen, etwa eines
anderen Mandanten, werden nie verwendet, auch nicht als Station zum Auffüllen oder Entladen.

## Eine Station anlegen

Über **Station anlegen** oben rechts öffnet sich ein Dialog. Trage einen Namen ein,
wähle die Organisation, der die Station gehört, und hake unter **Fahrzeuge können hier** an, was
dort geschieht. Die Position legst du auf der Karte fest: Verschiebe die Markierung an
den Standort der Station. Die Karte öffnet mit dem Kartenausschnitt deiner Organisation.
Nach **Speichern** erscheint die Station in der Liste.

## Eine Station bearbeiten oder löschen

Über das Stift-Symbol änderst du Name, Position und die Angabe **Fahrzeuge können hier**. Die
Organisation lässt sich hier nicht mehr wechseln. Das Papierkorb-Symbol löscht die
Station nach einer Rückfrage. Verweigert Green Ecolution das Löschen, etwa weil es sich
um den Standard handelt, erscheint der Grund als Meldung am Bildschirmrand.

![Der Dialog zum Bearbeiten einer Station mit Name, Kapazitäten und Position auf der Karte](../images/settings-station-form.png)

Über das Pfeil-Symbol **Verschieben** gibst du eine Station an eine andere Organisation ab;
sie dient dann deren Touren als Start, Ziel oder Station. Das Symbol erscheint nur, wenn
du Stationen in mindestens einer weiteren Organisation bearbeiten darfst, und nie bei der
Standardstation, weil jede Organisation genau eine davon hat.
