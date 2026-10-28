---
slug: admin-tenants
title: Mandantenübersicht
part: administration
summary: Was Mitglieder der obersten Organisation statt des Dashboards sehen, was die Mandantenübersicht zeigt und wie sie in die Ansicht eines Mandanten wechseln.
routes: ['/admin']
---

> [!NOTE]
> Diese Seite siehst du nur, wenn deine eigene Organisation die oberste Organisation der
> Instanz ist, also die Organisation Green Ecolution ganz oben im
> [Organisationsbaum](./settings-organization.md#der-organisationsbaum). Alle anderen
> landen nach der Anmeldung wie gewohnt auf dem [Dashboard](./dashboard.md).

## Wer die Übersicht sieht

Die oberste Organisation ist kein Mandant, sondern der Betreiber der Instanz. Wer ihr
angehört, arbeitet nicht mit Bäumen, Fahrzeugen und Einsatzplänen einer einzelnen
Kommune, sondern verwaltet die Organisationen, die das tun. Deshalb zeigt die Anwendung
solchen Mitgliedern eine eigene Oberfläche: Statt des Dashboards öffnet sich die
Mandantenübersicht, und die Seitennavigation führt unter **Instanz** zu den Seiten, die
die Instanz als Ganzes betreffen: Mandanten, Organisation, Team und Rollen, Plugins und
Systeminformationen. Karte, Einsatzpläne, Fahrzeuge und Sensoren tauchen dort nicht auf,
denn sie gehören immer zu einem bestimmten Mandanten. In der obersten Organisation sind
diese Seiten auch nicht erreichbar: Wer eine solche Adresse öffnet, die Zurück-Taste
benutzt oder aus einem Mandanten auf einer solchen Seite zurück in die eigene
Organisation wechselt, landet wieder in der Mandantenübersicht.

![Die Mandantenübersicht mit zwei Mandanten und ihren Kennzahlen](../images/admin-tenants.png)

## Was die Übersicht zeigt

Jede Karte steht für einen Mandanten, also eine Organisation direkt unterhalb der obersten.
Sie nennt die Zahl seiner Unterorganisationen, die Mitarbeitenden in seinem gesamten Ast,
die bei ihm aktiven Module sowie die Sensoren, Fahrzeuge und Einsatzpläne, die ihm und
seinen Unterorganisationen gehören. Archivierte Fahrzeuge zählen nicht mit. Solange die
Zahlen noch geladen werden, steht ein Strich an ihrer Stelle.

Hat die Instanz noch keinen Mandanten, legst du den ersten auf der Seite
[Organisation](./settings-organization.md#unterorganisationen) an; die Übersicht verlinkt
dorthin.

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
