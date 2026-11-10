---
slug: dashboard
title: Das Dashboard
part: intro
summary: Was das persönliche Dashboard zeigt, wie die Kennzahlen zu lesen sind und wie es von dort weitergeht.
routes: ['/dashboard']
---

## Was das Dashboard zeigt

Nach der Anmeldung landest du auf dem Dashboard, deiner persönlichen Übersichtsseite.
Sie begrüßt dich namentlich, zeigt die wichtigsten Kennzahlen deiner Organisation und
führt von dort direkt in die Bereiche, in denen etwas zu tun ist.

Eine Ausnahme gibt es: Gehört deine eigene Organisation ganz oben in den Organisationsbaum,
bist du also Mitglied der obersten Organisation der Instanz, landest du statt auf dem
Dashboard in der [Mandantenübersicht](./admin-tenants.md). Das Dashboard siehst du dann,
sobald du einen Mandanten geöffnet hast.

![Das persönliche Dashboard mit den Kennzahlen unter Auf einen Blick und den Schnellzugriffen darunter](../images/dashboard.png)

## Auf einen Blick

Der obere Bereich zeigt, was gerade Aufmerksamkeit braucht. Jede Kachel nennt eine Zahl
mit einer kurzen Einordnung darunter. Ein Klick führt zur passenden Liste:

- **Gruppen sehr trocken** zählt die Bewässerungsgruppen im Zustand **Sehr trocken**.
  Darunter steht, wie viele Gruppen leicht trocken sind und wie viele es insgesamt gibt.
  Der Klick öffnet die Gruppenliste, bereits auf sehr trockene Gruppen gefiltert.
- **Einsätze unterwegs** zählt die Einsatzpläne, die gerade laufen. Darunter steht, wie
  viele Einsätze für heute geplant sind. Der Klick öffnet das Einsatzplan-Board.
- **Sensoren offline** zählt die aktivierten Sensoren, die sich zu lange nicht gemeldet
  haben. Darunter steht die Zahl aller aktivierten Sensoren. Der Klick öffnet die
  Sensorliste mit dem Filter **Offline**.

Die Farbe einer Kachel folgt ihrer Zahl: Rot steht für etwas, das zu tun ist, Grün für
einen unauffälligen Stand. Eine Kachel erscheint nur, wenn du den Bereich dahinter lesen
darfst. Ist das Modul Bewässerung für deine Organisation abgeschaltet (siehe
[Organisation](./settings-organization.md#module)), fehlt die Kachel zu den
Bewässerungsgruppen.

## Schnellzugriff

Darunter liegen kompakte Verweise auf die am häufigsten gebrauchten Bereiche: **Karte**,
**Einsatzpläne**, **Sensoren**, **Einstellungen** und **Eigenes Profil**, jeweils mit einer
Zeile Beschreibung. Karte und Einsatzpläne fehlen, solange kein Modul aktiv ist, das sie
nutzt. **Einstellungen** und **Eigenes Profil** stehen jeder und jedem offen. Für die
übrigen Bereiche entscheiden deine Rollen; fehlt die Berechtigung, meldet dir die
Anwendung beim Öffnen, dass der Zugriff fehlt.

## Von hier aus weiter

Für die vollständige Übersicht aller Bereiche nutzt du die
[Seitennavigation](./getting-started.md#seitennavigation); über den Navigationspfad in
der Kopfzeile kommst du von jeder tieferen Seite mit einem Klick auf **Dashboard**
wieder hierher zurück.
