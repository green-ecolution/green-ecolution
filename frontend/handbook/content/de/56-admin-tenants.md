---
slug: admin-tenants
title: Mandantenübersicht
part: administration
summary: Was Mitglieder der obersten Organisation statt des Dashboards sehen, was die Mandantenübersicht zeigt, wie sie Module je Mandant freigeben und sperren und wie sie in die Ansicht eines Mandanten wechseln.
routes: ['/admin']
---

> [!NOTE]
> Diese Seite siehst du nur, wenn deine eigene Organisation die oberste Organisation der
> Instanz ist, also die Organisation Green Ecolution ganz oben im
> [Organisationsbaum](./settings-structure.md#der-organisationsbaum). Alle anderen
> landen nach der Anmeldung wie gewohnt auf dem [Dashboard](./dashboard.md).

## Wer die Übersicht sieht

Die oberste Organisation ist kein Mandant, sondern der Betreiber der Instanz. Wer ihr
angehört, arbeitet nicht mit Bäumen, Fahrzeugen und Einsatzplänen einer einzelnen
Kommune, sondern verwaltet die Organisationen, die das tun. Deshalb zeigt die Anwendung
solchen Mitgliedern eine eigene Oberfläche: Statt des Dashboards öffnet sich die
Mandantenübersicht, und die Seitennavigation führt unter **Instanz** zu den Seiten, die
die Instanz als Ganzes betreffen: Mandanten, Organisation, Team & Rollen, Plugins und
Systeminformationen. Die Einträge **Organisation**, **Team & Rollen** und **Plugins**
führen dabei in die Einstellungen der obersten Organisation, also auf deren
[Überblick](./settings-organization.md), auf die Mitarbeitenden und auf die Plugins.
Karte, Einsatzpläne, Fahrzeuge und Sensoren tauchen dort nicht auf,
denn sie gehören immer zu einem bestimmten Mandanten. In der obersten Organisation sind
diese Seiten auch nicht erreichbar: Wer eine solche Adresse öffnet, die Zurück-Taste
benutzt oder aus einem Mandanten auf einer solchen Seite zurück in die eigene
Organisation wechselt, landet wieder in der Mandantenübersicht.

![Die Mandantenübersicht mit einem Mandanten, seinem Bestand und den Modulen](../images/admin-tenants.png)

## Was die Übersicht zeigt

Die Übersicht ist eine Tabelle mit einer Zeile je Mandant, also je Organisation direkt
unterhalb der obersten. Die Spalte **Mandant** nennt die Zahl seiner Unterorganisationen
und die Mitarbeitenden in seinem gesamten Ast. Die Spalte **Bestand** zählt die
Sensoren, Fahrzeuge und Einsatzpläne, die ihm und seinen Unterorganisationen gehören.
Archivierte Fahrzeuge zählen nicht mit. Solange die Zahlen noch geladen werden, steht ein
Strich an ihrer Stelle. Danach folgt für jedes Modul der Instanz eine eigene Spalte mit
dem Namen des Moduls und seinem technischen Schlüssel. Am Ende der Zeile steht der Knopf
**Als Mandant öffnen**.

Hat die Instanz noch keinen Mandanten, steht anstelle der Tabelle ein Hinweis mit dem
Link **Mandanten unter Organisation anlegen**. Er öffnet den
[Überblick](./settings-organization.md) der obersten Organisation. Von dort führt die
Karte **Struktur**, ebenso der Eintrag **Struktur** in der Einstellungsnavigation, zu der
Seite, auf der du den Mandanten als
[Unterorganisation](./settings-structure.md#unterorganisationen) der obersten Organisation
anlegst.

## Module freigeben und sperren

Die oberste Organisation legt je Mandant fest, welche Module er nutzen darf. In der
Spalte eines Moduls steht dafür pro Mandant ein Schalter mit dem Zustand
**Freigegeben** oder **Gesperrt**. Er wirkt sofort, ein gesonderter Speichern-Knopf ist
nicht nötig. Ein neu angelegter Mandant hat zunächst kein Modul freigegeben. Gibst du ein
Modul zum ersten Mal frei, schaltet die Anwendung es beim Mandanten gleich ein. Ob der
Mandant das Modul danach nutzt, entscheidet er selbst unter
[Aufgabenbereiche](./settings-areas.md); die Zeile unter dem Schalter nennt den Stand
**Vom Mandanten eingeschaltet** oder **Vom Mandanten ausgeschaltet**. Ein gesperrtes
Modul steht dort mit dem Hinweis „Für TBZ nicht sichtbar“, wobei statt TBZ der Name des
Mandanten steht.

Sperren wirkt für den Mandanten und alle seine Unterorganisationen. Das Modul
verschwindet dort aus der Navigation und aus den Aufgabenbereichen, und seine Seiten
sind nicht mehr erreichbar. Die vorhandenen Daten bleiben gespeichert. Auch der
Schalter des Mandanten bleibt, wie er war: Gibst du das Modul später wieder frei,
findet der Mandant ihn so vor, wie er ihn verlassen hat, und die Daten sind wieder da.
Nutzt der Mandant das Modul gerade, fragt die Anwendung vor dem Sperren nach und nennt,
was verschwindet und dass die Daten bleiben.

Freigaben ändern kann, wer in der obersten Organisation Einstellungen bearbeiten darf
(`setting:update`). Wer dort Einstellungen nur lesen darf, sieht die Tabelle, kann die
Schalter aber nicht umlegen.

## In einen Mandanten wechseln

Mit **Als Mandant öffnen** wechselst du in die Organisation des Mandanten und siehst die
Anwendung so, wie seine Mitarbeitenden sie sehen: das Dashboard, die Karte, die Pläne und
alles Weitere, zugeschnitten auf diesen Mandanten. Technisch ist das derselbe
Organisationswechsel, den du auch über das Benutzermenü auslöst (siehe
[Erste Schritte](./getting-started.md), Abschnitt "Organisation wählen"); die
Benutzerkarte unten in der Seitennavigation zeigt dann den Namen des Mandanten statt
deiner E-Mail-Adresse.
Zurück in die Übersicht kommst du über denselben Weg: Wähle im Benutzermenü unter
**Organisation** wieder deine eigene Organisation, oder nutze **Zu meiner Organisation
wechseln**, auf den Fehlerseiten.
