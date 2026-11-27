---
slug: settings-team
title: Mitarbeitende und Rollen
part: administration
summary: Mitarbeitende verwalten und hinzufügen, Rollen und ihren Zuschnitt, den Berechtigungskatalog aus Ressource und Aktion sowie die mitgelieferten Rollenvorlagen.
routes: ['/settings/organization/$orgId/members', '/settings/organization/$orgId/roles']
---

> [!NOTE]
> Mitarbeitende einzusehen setzt die Berechtigung `user:read` voraus, Rollen einzusehen
> die Berechtigung `role:read`. Fehlt dir eine davon in der gewählten Organisation,
> taucht der zugehörige Eintrag **Mitarbeitende** oder **Rollen** in deiner
> Einstellungsnavigation gar nicht erst auf. Wende dich an die Verwaltung deiner
> Organisation, wenn du Zugriff brauchst.

## Mitarbeitende verwalten

Beide Seiten, **Mitarbeitende** und **Rollen**, sind eigene Einträge der
Einstellungsnavigation und beziehen sich auf die Organisation, die du dort oben gewählt
hast. Die Seite **Mitarbeitende** listet links die Personen dieser Organisation,
durchsuchbar nach Name, Username oder E-Mail-Adresse und einschränkbar nach Rolle.
Hat die Organisation Unterorganisationen, steht über der Liste der Schalter **mit
Unterorganisationen**. Er ist zunächst eingeschaltet, die Liste zeigt dann auch die
Personen der Unterorganisationen; schaltest du ihn aus, bleiben nur die Personen übrig,
die der gewählten Organisation selbst zugeordnet sind. Rechts zeigt die Seite die Details
der ausgewählten Person in drei Bereichen:
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

## Eine Person hinzufügen

Mit der Berechtigung `user:create` in der gewählten Organisation legst du neue
Mitarbeitende direkt in der Anwendung an. Die Schaltfläche **Person hinzufügen** steht in
der Kopfzeile der Seite, bei einer leeren Organisation stattdessen in der erklärenden
Ansicht. Der Dialog fragt nach Vorname, Nachname, E-Mail-Adresse und Benutzername; diese
vier Angaben sind Pflicht und mit einem Sternchen markiert. Der
Benutzername ist mit dem Teil der E-Mail-Adresse vor dem @ vorbelegt und lässt sich
ändern; hast du ihn einmal von Hand angepasst, folgt er der E-Mail-Adresse nicht mehr.
Mit diesem Benutzernamen meldet sich die Person an. Rollen sind freiwillig und lassen sich
auch später vergeben, wie oben unter
[Mitarbeitende verwalten](./settings-team.md#mitarbeitende-verwalten) beschrieben. Zur
Auswahl stehen nur Rollen der gewählten Organisation.

![Der Dialog Person hinzufügen mit ausgefüllten Angaben und zwei gewählten Rollen](../images/settings-members-add.png)

Die Anwendung legt das Konto selbst an und vergibt ein Startpasswort. Du siehst es genau
einmal im Dialog und kannst es dort kopieren; die Anwendung verschickt keine E-Mail.
Gib das Passwort deshalb persönlich weiter. Die Person muss es bei der ersten Anmeldung
ändern. Nach **Fertig** erscheint sie in der Liste und ist dort ausgewählt. Ist der
Benutzername oder die E-Mail-Adresse bereits vergeben, steht eine Meldung oberhalb beider
Felder. Welches der beiden Felder betroffen ist, meldet das System nicht getrennt, prüfe
also beide. Auf einer Instanz ohne Anmeldung wird die Funktion nicht angeboten.

Das Startpasswort kann die Anwendung weder ein zweites Mal anzeigen noch neu vergeben.
Solange der Dialog es zeigt, schließt er sich deshalb nur über **Fertig** oder das
Schließen-Symbol, und bevor du die Seite verlässt, fragt die Anwendung nach. Geht das
Passwort trotzdem verloren, oder wurde das Hinzufügen unterbrochen und meldet der Dialog
den Benutzernamen oder die E-Mail-Adresse als vergeben, obwohl die Person nicht in der
Liste steht, lässt sich das in der Anwendung nicht beheben. Dann muss der Betreiber der
Instanz das Konto im Anmeldedienst zurücksetzen oder entfernen.

## Wenn noch niemand da ist

Enthält die Liste mit Unterorganisationen, ohne Suche und ohne Rollenfilter, nur dich
selbst, zeigt die Seite statt der Liste eine erklärende Ansicht mit der Überschrift
„Außer dir ist noch niemand in“ und dem Namen der Organisation. Hat die Organisation gar
keine Person, lautet sie „In“, der Name und „ist noch niemand“. Die Ansicht erklärt, wer
Mitarbeitende sind, zeigt ein Beispiel und nennt drei Verwendungen: eigener Zugang,
Rollen vergeben und Personen für Einsätze einteilen. Dazu steht ein Hinweis, dass eine
Person immer zu einer Organisation gehört und sich später in eine andere verschieben
lässt. Wer `user:create` besitzt, findet hier **Person hinzufügen**. Stehst du selbst in
der Liste, erreichst du aus dieser Ansicht über **Eigenes Profil bearbeiten** dein eigenes
Profil. Sobald eine weitere Person dazukommt, erscheint die gewohnte Liste mit den Details
daneben. Suchst du oder filterst du nach Rolle, bleibt es bei der Meldung, dass nichts
gefunden wurde.

![Die Seite Mitarbeitende, solange nur die angemeldete Person in der Organisation ist](../images/settings-members-empty.png)

## Rollen und ihr Zuschnitt

Eine Rolle besteht aus einem Namen, einer optionalen Beschreibung und einer Menge von
Rechten, und sie gehört immer zu genau einer Organisation. Die Seite **Rollen** listet
die Rollen der gewählten Organisation in zwei Gruppen: **System · nicht editierbar** für die mitgelieferten
[Rollenvorlagen](./settings-team.md#rollenvorlagen) und **Eigene Rollen** für die Rollen dieser
Organisation. Eine unangetastete Kopie einer Vorlage sieht darin noch genauso aus wie
die Vorlage selbst und erscheint deshalb erst nach ihrer ersten Änderung als eigene
Rolle in der Liste; zuweisen lässt sie sich aber von Anfang an.

Mit der Berechtigung `role:create` legst du über **Neu** eine leere eigene Rolle in der
gewählten Organisation an
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

Die Rollenmatrix zeigt nur die Bereiche der Module, die in der gewählten Organisation
aktiv sind. Schaltet diese Organisation das Modul Bewässerung ab (siehe
[Aufgabenbereiche](./settings-areas.md#aufgabenbereiche-ein-und-ausschalten)), fehlen in
der Matrix die Bereiche Bäume, Bewässerungsgruppen und Einsatzpläne. Die bereits
gespeicherten Rechte dafür gehen dabei nicht verloren: Sie bleiben an der Rolle hängen
und stehen nach dem erneuten Einschalten des Moduls wieder zur Auswahl.

![Eine Rolle mit ihrer Rechteauswahl aus Kein, Ansehen, Bearbeiten und Verwalten je Bereich](../images/settings-team-roles.png)

## Der Berechtigungskatalog

Jedes Recht in Green Ecolution ist die Kombination aus einer Ressource und einer der
vier Aktionen Ansehen, Anlegen, Bearbeiten und Löschen, insgesamt 48 mögliche Rechte
über zwölf Ressourcen; die Zugriffsstufen aus dem vorigen Abschnitt setzen jeweils eine
feste Auswahl davon. In der Oberfläche sind diese Ressourcen zu drei Bereichen
gruppiert: **Grünflächen** (Bäume, Bewässerungsgruppen, Sensoren, Gebiete), **Planung**
(Einsatzpläne, Startpunkte, Fahrzeuge) und **Verwaltung** (Mitarbeitende, Organisation,
Rollen & Rechte, Plugins, Einstellungen). Die folgende Tabelle listet den vollständigen
Katalog, mit der technischen Bezeichnung, die auch in Fehlermeldungen der Anwendung
auftaucht. Startpunkte (Depots und Nachfüllstellen) haben eigene Rechte; wer bisher
Einsatzpläne bearbeiten durfte, darf auch Startpunkte bearbeiten. Für Einstellungen
werden nur Ansehen und Bearbeiten verwendet. Wer Einsatzpläne anlegen soll, braucht
zusätzlich `start_point:read`, weil das Formular einen Startpunkt verlangt und die
Auswahl nur Startpunkte anbietet, die die Rolle ansehen darf. Neu angelegte eigene
Rollen enthalten dieses Recht nicht von selbst.

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
| `start_point:read`     | Startpunkte ansehen           |
| `start_point:create`   | Startpunkt anlegen            |
| `start_point:update`   | Startpunkt bearbeiten         |
| `start_point:delete`   | Startpunkt entfernen          |
| `vehicle:read`         | Fahrzeuge ansehen             |
| `vehicle:create`       | Fahrzeug anlegen              |
| `vehicle:update`       | Fahrzeug bearbeiten           |
| `vehicle:delete`       | Fahrzeug löschen              |
| `region:read`          | Gebiete ansehen               |
| `region:create`        | Gebiet anlegen                |
| `region:update`        | Gebiet bearbeiten             |
| `region:delete`        | Gebiet löschen                |
| `user:read`            | Mitarbeitende ansehen         |
| `user:create`          | Mitarbeitende anlegen         |
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
| `plugin:read`          | Plugins ansehen               |
| `plugin:create`        | Plugin installieren           |
| `plugin:update`        | Plugin bearbeiten             |
| `plugin:delete`        | Plugin deinstallieren         |
| `setting:read`         | Einstellungen ansehen         |
| `setting:create`       | Nicht verwendet               |
| `setting:update`       | Einstellungen bearbeiten      |
| `setting:delete`       | Nicht verwendet               |

## Wie weit eine Rolle reicht

Eine Rolle gilt für ihre eigene Organisation und für den gesamten Ast darunter,
niemals für die Organisationen darüber; wie diese Richtung im Organisationsbaum
zustande kommt, steht in [Struktur](./settings-structure.md#wie-berechtigungen-im-baum-wirken).
Für die Zuweisung an eine Person gilt zusätzlich: Du brauchst die Berechtigung
`user:update` in der Organisation der Rolle, die Rechte der Rolle dürfen deine eigenen
nicht übersteigen, und die Rolle muss zur aktuellen Organisation der Person passen. Die
Seite **Rollen** zeigt und bearbeitet dabei immer nur die Rollen der gewählten
Organisation, auch **Kopieren & bearbeiten** legt eine Kopie stets dort an; eine neue
Rolle für eine andere Organisation legst du an, indem du in der Navigation diese
Organisation wählst. Gehört eine Person einer anderen Organisation als du, bleiben dir für
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
  Einsatzpläne, Fahrzeuge und Gebiete sowie auf die Organisationsstruktur mit ihren
  Aufgabenbereichen und ihren Werten, ohne Zugriff auf Mitarbeitende oder Rollen und
  ohne die Werte ändern zu können.
