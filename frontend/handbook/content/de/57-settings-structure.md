---
slug: settings-structure
title: Struktur
part: administration
summary: Wie der Organisationsbaum aufgebaut ist, was die Tabelle der Struktur zeigt, was die Seite ohne Unterorganisationen zeigt, wie sich eine Unterorganisation anlegen lässt und wie Berechtigungen im Baum nach unten wirken.
routes: ['/settings/organization/$orgId/structure']
---

> [!NOTE]
> Diese Seite setzt die Berechtigung `organization:read` voraus. Fehlt sie dir, taucht
> **Struktur** in deiner Einstellungsnavigation gar nicht erst auf; wende dich an die
> Verwaltung deiner Organisation, wenn du sie brauchst.

## Der Organisationsbaum

Jede Organisation in Green Ecolution ist ein Knoten in einem Baum: Sie hat höchstens
eine übergeordnete Organisation und beliebig viele untergeordnete. An der Organisation
hängt, wem eine Person, ein Baum, eine Bewässerungsgruppe, ein Sensor, ein Fahrzeug
oder ein Einsatzplan gehört, und über welche Rollen Zugriff darauf vergeben werden
kann. Genau eine Organisation steht ganz oben im Baum, sie trägt instanzweit immer den
Namen Green Ecolution und lässt sich nicht bearbeiten oder löschen.

Die Seite **Struktur** zeigt den Baum als Tabelle mit allen Organisationen, die du
sehen darfst. Untergeordnete Organisationen stehen eingerückt unter ihrer übergeordneten.
Jede Zeile nennt in der Spalte **Personen**, wie viele Mitarbeitende zur Organisation
gehören. Hat eine Organisation Unterorganisationen mit eigenen Mitarbeitenden, steht dort
die Gesamtzahl und dahinter, wie viele davon der Organisation direkt zugeordnet sind, etwa
„12, davon 4 direkt“. Die Spalte **Kontaktperson** zeigt die hinterlegte Person oder
**Nicht hinterlegt**, die Spalte **Module** die bei der Organisation eingeschalteten
Fachmodule oder **Keine**. Die Organisation, deren Einstellungen du gerade bearbeitest,
trägt unter ihrem Namen den Hinweis **Hier bist du gerade**.

Ein Klick auf den Namen einer Organisation öffnet ihren [Überblick](./settings-organization.md);
die Einstellungen aller weiteren Einträge der Navigation gelten ab dann für diese
Organisation. Dasselbe erreichst du über die Organisationswahl in der Navigation, die
unten als Eintrag **Struktur verwalten** hierher zurückführt.

![Die Struktur von TBZ mit ihren Unterorganisationen, Personenzahlen, Kontaktpersonen und Modulen](../images/settings-structure.png)

## Solange es keine Unterorganisationen gibt

Würde die Tabelle außer der gewählten Organisation keine weitere zeigen, ersetzt die Seite
sie durch eine erklärende Ansicht. Sie sagt, dass die gewählte Organisation noch keine
Unterorganisationen hat, und erklärt, was eine Unterorganisation ist: ein eigener Bereich
innerhalb der Organisation, etwa ein Betriebshof, eine Abteilung oder eine beauftragte
Firma. Ein kleines Beispiel zeigt, wie das aussehen könnte, und drei Punkte nennen den
Nutzen. Jede Unterorganisation hat ein eigenes Team mit eigenen Rollen, ihre Daten sind
getrennt, und die übergeordnete Organisation behält mit den passenden Rechten den
Überblick. Ein Hinweis erinnert daran, dass sich eine Unterorganisation später umbenennen,
aber nicht verschieben lässt und dass sie sich nur löschen lässt, solange sie weder
Mitarbeitende noch eigene Unterorganisationen hat. Wer `organization:create` besitzt,
findet in der Ansicht die Schaltfläche **Unterorganisation anlegen**. Sobald eine zweite
Organisation sichtbar ist, erscheint wieder die Tabelle, und die Schaltfläche steht in der
Kopfzeile der Seite. Ein Mitglied der obersten Organisation, das einen Mandanten ohne
Unterorganisationen wählt, sieht weiter die Tabelle, weil sie dann auch die übrigen
Organisationen zeigt.

![Die Struktur einer Organisation ohne Unterorganisationen mit Erklärung, Beispiel und Schaltfläche zum Anlegen](../images/settings-structure-empty.png)

## Unterorganisationen

Mit der Berechtigung `organization:create` legst du über **Unterorganisation anlegen**
eine neue Organisation unterhalb der gerade gewählten an; zunächst genügt ein Name,
Adresse und Kontaktperson trägst du anschließend im
[Überblick](./settings-organization.md) nach. Nach dem Anlegen öffnet die Anwendung
diesen Überblick für die neue Organisation. Der Name muss sich von seinen Geschwistern
unterscheiden, also von den anderen Organisationen mit derselben übergeordneten
Organisation; ein bereits vergebener Name wird zurückgewiesen.

Beim Anlegen bekommt eine neue Organisation sofort eigene, nutzbare Kopien aller
Rollenvorlagen; mehr dazu und wie sich diese Kopien zu eigenen Rollen entwickeln, steht
in [Mitarbeitende und Rollen](./settings-team.md#rollenvorlagen). Löschen lässt sich
eine Organisation im Überblick, und zwar nur, solange sie weder Unterorganisationen noch
zugeordnete Mitarbeitende mehr besitzt.

Ein neu angelegter Mandant startet ohne jedes Modul. Der Betreiber der Instanz gibt die
Module in der [Mandantenübersicht](./admin-tenants.md#module-freigeben-und-sperren) frei.

## Wie Berechtigungen im Baum wirken

Eine Rolle gehört immer zu genau einer Organisation und gilt für diese Organisation und
den gesamten Ast darunter, niemals für die Organisationen darüber. Ist eine Rolle also
einer weiter oben stehenden Organisation zugeordnet, reicht sie automatisch in jede ihr
untergeordnete Organisation hinein; eine Rolle einer untergeordneten Organisation
bleibt dagegen auf deren eigenen Ast beschränkt und erreicht weder Geschwister- noch
übergeordnete Organisationen. Deshalb liegt die Verwaltung, die über mehrere
Organisationen hinweg zuständig sein soll, sinnvollerweise weiter oben im Baum. Wie
Rollen im Einzelnen zugeschnitten werden und welche Rechte sich darin kombinieren
lassen, steht in [Mitarbeitende und Rollen](./settings-team.md).

Die Navigation der Einstellungen richtet sich ebenfalls nach diesem Prinzip: Deine Rechte
werden für die gewählte Organisation ausgewertet, und Einträge, für die du dort kein
Recht hast, fehlen. Dieselbe Person kann deshalb in einer Organisation mehr Einträge
sehen als in einer anderen.
