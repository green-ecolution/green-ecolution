---
slug: map
title: Die Karte
part: greenspaces
summary: Bäume und Bewässerungsgruppen auf der Karte lesen, sich orientieren und beides direkt dort erfassen.
routes: ['/map', '/map/tree/new', '/map/treecluster/new']
---

## Die Karte lesen

Die Karte zeigt alle Bäume und Bewässerungsgruppen in deinem Zuständigkeitsbereich an
ihrem tatsächlichen Standort. Beim Herauszoomen fasst sie benachbarte Bäume zu
Gruppensymbolen zusammen; klickst du eines an oder zoomst weiter hinein, löst es sich
in die einzelnen Baumsymbole auf. Jede Gruppe erscheint zusätzlich als eingefärbte
Umrandung um ihre Bäume, ein Klick auf die Fläche öffnet dieselbe Ansicht wie ein Klick
auf das Gruppensymbol.

Ein Klick auf einen Baum führt direkt zu dessen [Detailseite](./trees.md). Ein Klick auf
eine Gruppe öffnet stattdessen eine Seitenansicht mit Adresse, Baumzahl und den
häufigsten Baumarten, den vier Kennzahlen **Bewässerungszustand**, **Bodentemperatur**,
**Letzte Messung** und **Letzte Bewässerung**, sowie einer Vorschau der enthaltenen
Bäume; Bäume mit Sensor sind darin als **Sensor-Baum** markiert, die übrigen mit dem
Hinweis **kein Sensor**. Über **Zum Dashboard** gelangst du von dort zum vollständigen
[Gruppendashboard](./treecluster.md), über das Stiftsymbol daneben direkt in die
Bearbeitung. Oberhalb der Karte filterst du die Anzeige über die Suche nach einer Gruppe
und den Zustand der Bewässerung; wer eine Gruppe anlegen darf, findet dort auch die
Schaltfläche **Gruppe anlegen**.

## Ebenen und Legende

Über die Schaltfläche mit dem Fragezeichen unten links blendest du die Legende
**Bewässerungszustand** ein und wieder aus. Sie zeigt dieselben fünf Zustände, die auch
Baum- und Gruppensymbole einfärben:

| Zustand          | Bedeutung                                                      |
| ---------------- | -------------------------------------------------------------- |
| In Ordnung       | Die Bewässerung ist ausreichend, keine Maßnahmen erforderlich. |
| Leicht trocken   | Die Bäume sind leicht trocken und benötigen etwas Wasser.      |
| Sehr trocken     | Die Bäume benötigen dringend Wasser.                           |
| Soeben bewässert | Die Bäume wurden vor kurzem bewässert.                         |
| Unbekannt        | Der Bewässerungsstatus ist unbekannt.                          |

Die ersten drei Zustände bilden die eigentliche Ampel von grün über gelb nach rot, die
beiden übrigen sind neutral eingefärbt. Wie ein Baum und eine Gruppe zu ihrem jeweiligen
Zustand kommen, erklärt das [Gruppendashboard](./treecluster.md#bewasserungsstatus-und-wie-er-zustande-kommt).

## Die 3D-Ansicht

Die Schaltfläche **3D** in der Kartensteuerung rechts unten kippt die Karte in eine
perspektivische Ansicht und schaltet auf erneuten Klick wieder in die flache 2D-Ansicht
zurück. Hast du die Karte zusätzlich gedreht, richtet sie die Kompassschaltfläche
darüber mit einem Klick wieder exakt nach Norden aus und hebt die Kippung dabei
gleichzeitig auf.

## Der eigene Standort

Über die Schaltfläche **Eigenen Standort anzeigen** bittet die Anwendung den Browser um
deinen aktuellen Standort und zeigt ihn als Punkt samt Genauigkeitskreis auf der Karte
an; die Kartenansicht folgt deiner Position, bis du selbst an eine andere Stelle
scrollst oder ziehst. Ein erneuter Klick auf dieselbe, jetzt aktive Schaltfläche beendet
die Standortanzeige wieder.

> [!NOTE]
> Unterstützt der Browser keine Standortbestimmung oder liegt die ermittelte Position
> außerhalb des dargestellten Kartenbereichs, meldet dir die Anwendung das direkt und
> schaltet die Standortanzeige wieder aus.

## Bäume und Gruppen direkt auf der Karte erfassen

Einen neuen Baum legst du über die Schaltfläche mit dem Zahnrad-Symbol
(**Kataster-Einstellungen**) oben in der Werkzeugleiste an: Sie öffnet einen Dialog, aus
dem heraus du über **Neuen Baum manuell hinzufügen** zur Erfassung wechselst. Klicke
anschließend auf die Karte, um den Standort des Baums festzulegen; den gesetzten Marker
kannst du danach noch beliebig verschieben. Trage dann Baumnummer, Baumart, Pflanzjahr
und optional eine Bewässerungsgruppe sowie einen Sensor ein und speichere.

Eine neue Bewässerungsgruppe legst du über die Schaltfläche **Gruppe anlegen** an. Nach
Name, Adresse und Bodenbeschaffenheit klickst du die Bäume, die zur Gruppe gehören
sollen, einzeln auf der Karte an; ein erneuter Klick entfernt einen Baum wieder aus der
Auswahl. Blass dargestellte Bäume gehören einer anderen Organisation und lassen sich
einer neuen Gruppe nicht ohne Weiteres hinzufügen, dazu mehr im
[Kapitel zu den Bewässerungsgruppen](./treecluster.md#baume-zuordnen).

Das Bearbeiten eines Baums oder einer bestehenden Gruppe läuft nach demselben Muster:
Einen Baum rufst du dazu über dessen Detailseite oder direkt über sein Symbol auf der
Karte auf, eine Gruppe über das Stiftsymbol in ihrer Seitenansicht. In beiden Fällen
öffnet sich dieselbe Eingabemaske wie beim Anlegen, bereits mit den vorhandenen Werten
gefüllt.
