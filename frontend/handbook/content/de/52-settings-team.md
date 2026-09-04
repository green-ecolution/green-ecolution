---
slug: settings-team
title: Team und Rollen
part: administration
summary: Mitarbeitende, Rollen und ihren Zuschnitt verwalten, den Berechtigungskatalog aus Ressource und Aktion sowie die mitgelieferten Rollenvorlagen.
routes: ['/settings/team', '/settings/team/members', '/settings/team/roles']
---

> [!NOTE]
> Mitarbeitende einzusehen setzt die Berechtigung `user:read` voraus, Rollen einzusehen
> die Berechtigung `role:read`. Fehlen dir beide, taucht **Team & Rollen** in deiner
> Einstellungsnavigation gar nicht erst auf; fehlt dir nur eine davon, bleibt der
> jeweilige Reiter verborgen. Wende dich an die Verwaltung deiner Organisation, wenn du
> Zugriff brauchst.

## Mitarbeitende verwalten

Die Seite **Mitarbeitende** listet links alle Personen der Instanz, durchsuchbar nach
Name, Username oder E-Mail-Adresse und einschränkbar nach Organisation und Rolle;
rechts zeigt sie die Details der ausgewählten Person in drei Bereichen:
**Organisation**, **Rollen** und die persönlichen Daten. Zum Ändern braucht jeder
Bereich zusätzlich zum bloßen Ansehen die Berechtigung `user:update`.

Unter **Organisation** siehst du, welcher Organisation die Person angehört. Mit
`user:update` in der Zielorganisation und `organization:read`, um die Auswahl zu
füllen, kannst du sie wechseln; der Wechsel wirkt sofort, nach Bestätigung eines
Dialogs. Zugewiesene Rollen bleiben dabei bestehen, auch wenn sie zu einer anderen
Organisation gehören als die, in die die Person gerade gewechselt ist; sie gelten dort
unverändert weiter, denn eine Rolle richtet sich nach ihrer eigenen Organisation, nicht
nach der Organisation ihrer Inhaberin oder ihres Inhabers. Ist eine Person also
plötzlich in Bereichen unterwegs, die zu ihrer aktuellen Organisation gar nicht zu
passen scheinen, lohnt der Blick auf ihre Rollen und deren jeweilige Organisation.

![Die Mitarbeitendenliste mit ausgewählter Person und geöffneter Rollenzuweisung](../images/settings-team-members.png)

Unter **Rollen** siehst du die zugewiesenen Rollen der Person. Mit `user:update` und
zusätzlich `role:read`, um die zuweisbaren Rollen zu laden, weist du ihr eine Rolle aus
deren eigener Organisation zu oder entziehst ihr eine bereits zugewiesene; das wirkt
sofort, ohne eigenes Speichern. Zur Auswahl stehen nur Rollen der Organisation, der die
Person gerade angehört, denn eine Rolle lässt sich nur Mitgliedern der Organisation
zuweisen, der sie selbst gehört. Du kannst außerdem nur Rechte weitergeben, die du
selbst besitzt; enthält die Rolle mehr, weist die Anwendung die Zuweisung zurück. Mehr
zu beidem steht weiter unten unter
[Wie weit eine Rolle reicht](./settings-team.md#wie-weit-eine-rolle-reicht).

Die persönlichen Daten darunter, darunter Verfügbarkeit, Führerscheinklassen,
Telefonnummer, Personalnummer und ob die Person **Für Einsatzpläne auswählbar** ist,
ändern sich erst mit **Speichern**; nur wer dieses Feld gesetzt hat, erscheint bei der
Zuordnung von Mitarbeitenden zu einem [Einsatzplan](./watering-plans.md).

An deinem eigenen Konto lassen sich Organisation und Rollen über diese Seite nicht
ändern, selbst mit der passenden Berechtigung nicht; das muss eine andere Person mit
Verwaltungsrechten für dich übernehmen.

## Rollen und ihr Zuschnitt

Eine Rolle besteht aus einem Namen, einer optionalen Beschreibung und einer Menge von
Rechten, und sie gehört immer zu genau einer Organisation. Die Seite **Rollen** listet
sie in zwei Gruppen: **System · nicht editierbar** für die mitgelieferten
[Rollenvorlagen](./settings-team.md#rollenvorlagen) und **Eigene Rollen** für die Rollen deiner
Organisation. Eine unangetastete Kopie einer Vorlage sieht darin noch genauso aus wie
die Vorlage selbst und erscheint deshalb erst nach ihrer ersten Änderung als eigene
Rolle in der Liste; zuweisen lässt sie sich aber von Anfang an.

Mit der Berechtigung `role:create` legst du über **Neu** eine leere eigene Rolle an
oder über **Kopieren & bearbeiten** an einer bestehenden Rolle oder Vorlage eine Kopie
mit denselben Rechten, die du anschließend anpasst. Übersteigen die kopierten Rechte
deine eigenen, entfernt die Anwendung die überzähligen automatisch und weist dich
darauf hin, wie viele das waren. Rollenvorlagen selbst lassen sich nicht bearbeiten
oder löschen; ein Hinweis auf der jeweiligen Vorlage verlinkt das Wort **Kopiere**
direkt zu **Kopieren & bearbeiten**.

Beim Bearbeiten einer eigenen Rolle zeigt jeder Bereich eine Zugriffsstufe mit den
Optionen **Kein**, **Ansehen**, **Bearbeiten** und **Verwalten**: **Ansehen** setzt nur
das Leserecht, **Bearbeiten** zusätzlich Anlegen und Bearbeiten, aber nicht Löschen,
und erst **Verwalten** schließt auch das Löschen ein. Über den Pfeil neben der
Zugriffsstufe lässt sich der Bereich aufklappen und jedes der vier Rechte einzeln
setzen; passt die daraus entstehende Kombination zu keiner der vier Stufen, zeigt die
Anwendung stattdessen **Individuell**. Ein Recht, das du selbst nicht besitzt, lässt
sich dabei nicht aktivieren.

![Eine Rolle mit ihrer Rechteauswahl aus Kein, Ansehen, Bearbeiten und Verwalten je Bereich](../images/settings-team-roles.png)

## Der Berechtigungskatalog

Jedes Recht in Green Ecolution ist die Kombination aus einer Ressource und einer der
vier Aktionen Ansehen, Anlegen, Bearbeiten und Löschen, insgesamt 36 mögliche Rechte
über neun Ressourcen; die Zugriffsstufen aus dem vorigen Abschnitt setzen jeweils eine
feste Auswahl davon. In der Oberfläche sind diese Ressourcen zu drei Bereichen
gruppiert: **Grünflächen** (Bäume, Bewässerungsgruppen, Sensoren, Gebiete), **Planung**
(Einsatzpläne, Fahrzeuge) und **Verwaltung** (Mitarbeitende, Organisation, Rollen &
Rechte). Die folgende Tabelle listet den vollständigen Katalog, mit der technischen
Bezeichnung, die auch in Fehlermeldungen der Anwendung auftaucht.

| Berechtigung           | Bedeutung                     |
| ---------------------- | ----------------------------- |
| `tree:read`            | Bäume ansehen                 |
| `tree:create`          | Baum anlegen                  |
| `tree:update`          | Baum bearbeiten               |
| `tree:delete`          | Baum löschen                  |
| `tree_cluster:read`    | Bewässerungsgruppen ansehen   |
| `tree_cluster:create`  | Bewässerungsgruppe anlegen    |
| `tree_cluster:update`  | Bewässerungsgruppe bearbeiten |
| `tree_cluster:delete`  | Bewässerungsgruppe löschen    |
| `sensor:read`          | Sensoren ansehen              |
| `sensor:create`        | Sensor anlegen                |
| `sensor:update`        | Sensor bearbeiten             |
| `sensor:delete`        | Sensor löschen                |
| `watering_plan:read`   | Einsatzpläne ansehen          |
| `watering_plan:create` | Einsatzplan anlegen           |
| `watering_plan:update` | Einsatzplan bearbeiten        |
| `watering_plan:delete` | Einsatzplan löschen           |
| `vehicle:read`         | Fahrzeuge ansehen             |
| `vehicle:create`       | Fahrzeug anlegen              |
| `vehicle:update`       | Fahrzeug bearbeiten           |
| `vehicle:delete`       | Fahrzeug löschen              |
| `region:read`          | Gebiete ansehen               |
| `region:create`        | Gebiet anlegen                |
| `region:update`        | Gebiet bearbeiten             |
| `region:delete`        | Gebiet löschen                |
| `user:read`            | Mitarbeitende ansehen         |
| `user:create`          | Mitarbeitende einladen        |
| `user:update`          | Mitarbeitende bearbeiten      |
| `user:delete`          | Mitarbeitende entfernen       |
| `organization:read`    | Organisation ansehen          |
| `organization:create`  | Organisation anlegen          |
| `organization:update`  | Organisation bearbeiten       |
| `organization:delete`  | Organisation löschen          |
| `role:read`            | Rollen ansehen                |
| `role:create`          | Rolle anlegen                 |
| `role:update`          | Rolle bearbeiten              |
| `role:delete`          | Rolle löschen                 |

## Wie weit eine Rolle reicht

Eine Rolle gilt für ihre eigene Organisation und für den gesamten Ast darunter,
niemals für die Organisationen darüber; wie diese Richtung im Organisationsbaum
zustande kommt, steht in [Organisation](./settings-organization.md#wie-berechtigungen-im-baum-wirken).
Für die Zuweisung an eine Person gilt zusätzlich: Du brauchst die Berechtigung
`user:update` in der Organisation der Rolle, die Rechte der Rolle dürfen deine eigenen
nicht übersteigen, und die Rolle muss zur aktuellen Organisation der Person passen. Die
Seite **Rollen** zeigt und bearbeitet dabei immer nur die Rollen deiner eigenen
Organisation, auch **Kopieren & bearbeiten** legt eine Kopie stets dort an; eine neue,
eigens zugeschnittene Rolle für eine andere Organisation lässt sich über diese Seite
nicht anlegen. Gehört eine Person einer anderen Organisation als du, bleiben dir für
sie deshalb die Rollen, die diese Organisation bereits besitzt, insbesondere die
[Rollenvorlagen](./settings-team.md#rollenvorlagen), die schon bei ihrer Anlage jede
Organisation automatisch bekommt. Die eigene Rollenzuweisung lässt sich, wie die eigene
Organisation, nicht über diese Seite ändern.

## Rollenvorlagen

Jede Organisation bekommt bei ihrer Anlage automatisch eine eigene, sofort nutzbare
Kopie jeder Rollenvorlage; Änderungen an einer Kopie wirken sich nie auf die Vorlage
selbst aus, die als unveränderliche Ausgangsbasis erhalten bleibt. Green Ecolution
liefert fünf Vorlagen mit:

- **Administrator**: voller Zugriff auf alle Ressourcen.
- **Baumpflege**: Bäume und Bewässerungsgruppen vollständig verwalten, Sensoren und
  Gebiete ansehen.
- **Sensorik**: Sensoren vollständig verwalten, Bäume und Bewässerungsgruppen ansehen.
- **Routenplanung**: Einsatzpläne und Fahrzeuge vollständig verwalten, Bäume,
  Bewässerungsgruppen, Sensoren und Gebiete ansehen.
- **Beobachter**: lesender Zugriff auf Bäume, Bewässerungsgruppen, Sensoren,
  Einsatzpläne, Fahrzeuge und Gebiete, ohne Zugriff auf Mitarbeitende, Organisation
  oder Rollen.
