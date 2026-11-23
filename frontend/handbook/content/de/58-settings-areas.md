---
slug: settings-areas
title: Aufgabenbereiche
part: administration
summary: Welche Aufgabenbereiche eine Organisation nutzt und mit welchen Werten die Planung rechnet, vom Wasserbedarf je Baum bis zur Frist, nach der ein Sensor als offline gilt.
routes: ['/settings/organization/$orgId/areas']
---

> [!NOTE]
> Diese Seite setzt die Berechtigung `setting:read` voraus. Fehlt sie dir, taucht
> **Aufgabenbereiche** in deiner Einstellungsnavigation gar nicht erst auf. Zum Ändern
> brauchst du zusätzlich `setting:update`; ohne sie siehst du Schalter und Werte nur.

Auf dieser Seite legst du fest, was zu den Aufgaben einer Organisation gehört und mit
welchen Werten die Anwendung für sie rechnet. Jeder Aufgabenbereich hat eine eigene
Karte, darunter folgt die Karte **Sensoren**. Der Knopf **Speichern** am Ende der Seite
übernimmt alle Änderungen gemeinsam; bis dahin gilt, was vorher galt.

![Die Seite Aufgabenbereiche mit den Karten Bewässerung, Abfall und Sensoren](../images/settings-organization-values.png)

## Aufgabenbereiche ein- und ausschalten

Green Ecolution besteht aus einem gemeinsamen Kern und Aufgabenbereichen. Der Kern
umfasst die Organisationen, Rollen, Sensoren, Fahrzeuge und Stationen. Ein
Aufgabenbereich bringt mit, was zu einer bestimmten Aufgabe gehört: **Bewässerung**
liefert Bäume, Bewässerungsgruppen, Einsatzpläne und die Auswertung, **Abfall** die
Abfallbehälter und Leerungstouren.

Welche Karten erscheinen, bestimmt der Betreiber der Instanz: Er gibt jedem Mandanten
Module frei oder sperrt sie, siehe [Mandantenübersicht](./admin-tenants.md#module-freigeben-und-sperren).
Ein Aufgabenbereich, der nicht freigegeben ist, hat hier keine Karte.

Oben auf der Karte eines freigegebenen Aufgabenbereichs steht der Schalter mit dem
Zustand **Eingeschaltet** oder **Ausgeschaltet**. Ihn bedient der Mandant, also die
Organisation direkt unterhalb der obersten, ebenso die oberste Organisation selbst.
Unterorganisationen eines Mandanten sehen den Zustand ohne Schalter, dazu steht der
Hinweis „Diesen Aufgabenbereich schaltet die übergeordnete Organisation.“ Sie folgen
ihrem Mandanten. Die Werte unter einem eingeschalteten Aufgabenbereich kann dagegen
jede Organisation für sich eintragen.

Schaltest du einen Aufgabenbereich aus, verschwinden seine Seiten für alle
Mitarbeitenden der Organisation und ihrer Unterorganisationen aus der Navigation, von
der Karte, vom Dashboard und aus der Einsatzplanung. Wer einen Link auf eine solche
Seite öffnet, sieht stattdessen den Hinweis **Modul nicht aktiv**. Die Daten bleiben
unberührt und stehen wieder zur Verfügung, sobald der Aufgabenbereich erneut
eingeschaltet wird. Auch die Rechte, die Rollen dafür enthalten, bleiben gespeichert,
wirken aber nur, solange er eingeschaltet ist, siehe
[Mitarbeitende und Rollen](./settings-team.md#der-berechtigungskatalog). Diese Hilfe
richtet sich ebenfalls danach: Wer angemeldet ist, sieht nur die Kapitel zu
Aufgabenbereichen, die für ihn eingeschaltet sind. Ein direkt geöffneter Link auf ein
ausgeblendetes Kapitel funktioniert trotzdem.

## Werte

Unter einem eingeschalteten Aufgabenbereich stehen seine Werte, jeder als Feld mit
Einheit. Schaltest du einen Aufgabenbereich gerade erst ein, erscheinen seine Felder
sofort, und du kannst sie im selben Zug mit speichern.

- **Wasserbedarf je Baum** (Bewässerung, in Litern): So viel Wasser rechnet die
  Tourenplanung für jeden Baum ein. Der Wert bestimmt, wie viele Bewässerungsgruppen in
  eine Fahrt passen. Ein bereits berechneter Einsatzplan wird nicht rückwirkend
  umgerechnet.
- **Frisch gegossen gilt für** (Bewässerung, in Stunden): So lange zeigt ein Baum nach
  einem abgeschlossenen Einsatz den Zustand
  [Soeben bewässert](./treecluster.md#bewasserungsstatus-und-wie-er-zustande-kommt),
  bevor er auf **Unbekannt** zurückfällt.
- **Behälter gilt als voll ab** (Abfall, in Prozent): Ab diesem Füllstand zeigt die
  Anwendung einen Behälter als voll und schlägt ihn der Tourenplanung zur Leerung vor.
  Bereits geplante Touren bleiben unverändert.

Unter jedem Feld steht der übliche Wert, etwa „Üblich sind 80 Liter.“ Das ist der Wert,
der ohne eigene Eingabe gilt: der Wert der übergeordneten Organisation und ganz oben
der Standard der Anwendung. Trägst du den üblichen Wert ein, gilt deine Organisation
wieder als ohne eigene Eingabe. Ändert die übergeordnete Organisation ihren Wert
später, ändert er sich dann auch bei dir. Ein Wert außerhalb des erlaubten Bereichs
wird direkt am Feld erklärt, und **Speichern** bleibt so lange gesperrt.

## Sensoren

Die Karte **Sensoren** hat keinen Schalter, denn Sensoren gehören zum Kern.

- **Sensor gilt als offline nach** (in Stunden, 1 bis 720): Meldet sich ein Sensor so
  lange nicht, zeigt die Anwendung ihn als offline. Dieselbe Frist entscheidet, wie
  lange ein Messwert als aktuell gilt; danach wird zum Beispiel der Füllstand eines
  Abfallbehälters als unbekannt angezeigt.
- **Sensor gilt als auffällig nach** (in Messungen, 1 bis 100): So viele unbrauchbare
  Messungen in Folge braucht es, bis ein Sensor als auffällig markiert wird.

Auch hier steht unter jedem Feld der übliche Wert, und er verhält sich wie oben
beschrieben.
