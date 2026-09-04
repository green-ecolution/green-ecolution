---
slug: troubleshooting
title: Häufige Störungen
part: appendix
summary: Was zu tun ist, wenn die Anmeldung nicht klappt, eine Seite den Zugriff verweigert, ein Sensor stumm bleibt oder sich keine Route berechnen lässt.
routes: []
---

Dieses Kapitel sammelt die Störungen, die im laufenden Betrieb tatsächlich vorkommen. Zu
jeder steht, woran du sie erkennst, was üblicherweise dahintersteckt und wie du
weiterkommst. Bleibt eine Störung bestehen, hilft [Systeminformationen](./system-info.md)
dabei, sie so genau zu beschreiben, dass Verwaltung oder Support etwas damit anfangen
können.

## Anmeldung schlägt fehl

Green Ecolution führt die Anmeldung nicht selbst durch, sondern übergibt sie an den
Anmeldedienst deiner Organisation, siehe [Anmelden](./getting-started.md#anmelden). Meldet
dieser Dienst falsches Passwort oder unbekannten Benutzernamen zurück, ist das eine Sache
der Verwaltung deiner Organisation, nicht von Green Ecolution selbst; wende dich mit
Fragen zu deinem Konto dorthin.

Wirst du dagegen mitten in der Arbeit unvermittelt wieder auf die Anmeldeseite
geschickt, ist das kein Fehler: Deine Sitzung ist im Hintergrund abgelaufen, und die
Anwendung schickt dich automatisch zur erneuten Anmeldung, ohne dass dabei Daten
verloren gehen. Meldet dir stattdessen eine einzelne Aktion, die Anmeldung sei
abgelaufen, bitte melde dich erneut an, betrifft das nur diese eine Anfrage; melde dich
neu an und versuche die Aktion noch einmal. Bleibt die Seite dagegen komplett weiß oder
meldet, der Server sei nicht erreichbar, prüfe zuerst deine eigene Internetverbindung.

Kommst du zwar an, siehst danach aber in fast jedem Bereich **Kein Zugriff**, entscheidet
darüber ausschließlich, welche Rollen deinem Konto zugewiesen sind und zu welcher
Organisation jede dieser Rollen selbst gehört; die Organisation deines eigenen Kontos
spielt für diese Frage keine Rolle, siehe
[Wie weit eine Rolle reicht](./settings-team.md#wie-weit-eine-rolle-reicht). Der
wahrscheinlichste Grund für fehlenden Zugriff überall ist deshalb, dass dir noch gar
keine Rolle zugewiesen wurde; das prüft und behebt eine Person mit den nötigen Rechten
unter [Mitarbeitende verwalten](./settings-team.md#mitarbeitende-verwalten), im Bereich
**Rollen** der betroffenen Person.

Fehlt deinem Konto stattdessen die Organisation selbst, wirkt sich das an anderer Stelle
aus, nicht beim Ansehen bestehender Bereiche: Legst du eine Rolle an oder lädst eine neue
Mitarbeiterin oder einen neuen Mitarbeiter ein, verwendet die Anwendung dafür immer die
Organisation deines eigenen Kontos, ohne dass sich das umgehen ließe; fehlt sie, meldet
sie, dass keine Organisation angegeben wurde und dein Konto zu keiner gehört. Auch das
behebt nur eine Person mit den nötigen Rechten über
[Mitarbeitende verwalten](./settings-team.md#mitarbeitende-verwalten).

## Eine Seite meldet „Kein Zugriff"

Statt des erwarteten Inhalts zeigt die Seite ein Schloss-Symbol mit dem Titel
**Kein Zugriff** und dem Hinweis, dass für diesen Bereich die nötige Berechtigung fehlt.
Das ist die normale Reaktion darauf, dass deine Rollen die aufgerufene Seite nicht
abdecken, siehe [Rollen und Sichtbarkeit](./introduction.md#rollen-und-sichtbarkeit): Die
Anwendung prüft das schon, bevor sie überhaupt Daten dafür lädt, und meldet den
fehlenden Zugriff statt eines leeren oder halb geladenen Bereichs. Bereiche, für die dir
die Berechtigung fehlt, tauchen deshalb in der Seitennavigation gar nicht erst auf; nur
ein direkt aufgerufener Link führt noch auf diese Meldung.

Verweigert stattdessen nur eine einzelne Schaltfläche mit einer kurzen Meldung die
Aktion, obwohl die Seite selbst sich öffnet, fehlt dir statt der Ansicht nur das Recht
für genau diese eine Aktion, etwa Bearbeiten oder Löschen an einer Stelle, die du sonst
nur ansehen darfst.

In beiden Fällen hilft nur die Verwaltung deiner Organisation weiter: Sie ordnet dir die
passende Rolle zu oder schneidet eine bestehende passend zu, siehe
[Wie weit eine Rolle reicht](./settings-team.md#wie-weit-eine-rolle-reicht). Prüft die
Verwaltung eine Zuweisung nach, lohnt sich zusätzlich der Blick in
[Wie Berechtigungen im Baum wirken](./settings-organization.md#wie-berechtigungen-im-baum-wirken):
Eine Rolle wirkt nur nach unten im Organisationsbaum, nie nach oben.

## Ein Sensor bleibt offline

Auf der Detailseite oder in der Geräteliste steht ein Sensor dauerhaft auf **Offline**,
auch nach mehreren Tagen. Das bedeutet zunächst nur, dass seit über 24 Stunden keine
Übertragung eingegangen ist, siehe
[Die Verbindungszustände: vorbereitet, online und offline](./sensors.md#die-verbindungszustande-vorbereitet-online-und-offline);
der Sensor selbst meldet nicht, ob er in Ordnung ist, die Anwendung leitet den Zustand
ausschließlich aus der Zeit seit der letzten Übertragung ab. Ein als offline
angezeigter Sensor ist deshalb nicht zwangsläufig defekt.

Prüfe zuerst auf der Detailseite **Akkustand** und **Letztes Signal**: Fällt die
Batteriespannung unter 2,8 V, schaltet sich der Sensor selbst ab, und ein Wechsel oder
eine Aufladung der Batterie behebt die Störung. War das Signal zuvor gut und bricht
plötzlich ab, deutet das eher auf ein Funkloch, eine Beschädigung am Standort oder ein
verschobenes Gateway hin; das lässt sich von der Anwendung aus nicht unterscheiden,
sondern nur vor Ort prüfen.

Zeigt ein Gerät stattdessen dauerhaft **Vorbereitet**, ist es noch nicht aktiviert und
deshalb noch keinem Baum zugeordnet; das ist kein Störungsfall, sondern der Ausgangszustand
vor der Aktivierung, siehe [Sensor aktivieren](./sensors.md#sensor-aktivieren).

## Messwerte fehlen

Auf der Sensor-Detailseite oder im Bereich **Wasserversorgung** eines Gruppendashboards
fehlt ein Verlauf, wo eigentlich einer stehen sollte. Dafür kommen mehrere, klar
unterscheidbare Ursachen infrage.

Ist der Sensor noch nicht aktiviert, liefert er naturgemäß keine Messwerte, weil er
keinem Baum zugeordnet ist. Ist er aktiviert, aber offline, fehlen aktuelle Werte aus
demselben Grund wie im vorigen Abschnitt beschrieben. Zeigt die Detailseite dagegen
Werte, aber daneben einen Hinweis zur **Datenqualität**, liegen durchaus Messwerte vor,
die Anwendung hat in den letzten sieben Tagen jedoch wiederholt unplausible Werte
erkannt; das betrifft die Verlässlichkeit der Daten, nicht die Verbindung, siehe
[Messwerte und Signalqualität auf der Detailseite](./sensors.md#messwerte-und-signalqualitat-auf-der-detailseite).

Fehlt der Verlauf stattdessen im Bereich **Wasserversorgung** einer ganzen Gruppe, obwohl
Sensoren verbaut sind, prüfe die Bodenart der Gruppe: Sensoren mit volumetrischer
Bodenfeuchtemessung liefern ohne bekannte Bodenbeschaffenheit keinen Bewässerungszustand,
siehe [Bodenbeschaffenheit](./treecluster.md#bodenbeschaffenheit). Welche weiteren Gründe
für einen unbekannten Bewässerungszustand infrage kommen, listet
[Bewässerungsstatus und wie er zustande kommt](./treecluster.md#bewasserungsstatus-und-wie-er-zustande-kommt)
vollständig auf.

## Eine Route lässt sich nicht berechnen

Nach dem Speichern eines Einsatzplans bleibt der Reiter **Route** aus, und die
Detailseite weist darauf hin, dass sich keine Route berechnen ließ. Meistens liegt das
daran, dass die ausgewählten Bewässerungsgruppen zusammen mehr Wasser benötigen, als
das gewählte Fahrzeug samt einem eventuellen Anhänger fasst; prüfe die Wasserkapazität
gegen die Anzahl der Bäume in den gewählten Gruppen, siehe
[Route festlegen](./watering-plans.md#route-festlegen). Ebenso führt ein fehlender
Startpunkt oder ein fehlendes Fahrzeug dazu, dass sich aus den restlichen Angaben keine
Route ergibt.

Ist die Auswahl an Fahrzeug, Startpunkt und Gruppen dagegen plausibel und die
Berechnung schlägt trotzdem fehl, kann der Routing-Dienst selbst gerade nicht
erreichbar sein oder für diese Instanz gar nicht aktiviert sein. Ein Blick auf
[Systeminformationen](./system-info.md#was-die-systeminformationen-zeigen) im Reiter
**System** zeigt, ob der Dienst Routing als **Deaktiviert** oder mit einem
Verbindungsfehler geführt wird; ist er deaktiviert, lässt sich für diese Instanz
grundsätzlich keine Route berechnen, unabhängig von der Auswahl im Einsatzplan. In
beiden Fällen bleibt der Einsatzplan selbst erhalten, nur ohne Route; ein erneutes
Speichern nach Behebung der Ursache löst die Berechnung neu aus.

## Die Karte bleibt leer

Die Kartenseite öffnet sich, zeigt aber weder Bäume noch Bewässerungsgruppen, oder der
Kartenhintergrund selbst bleibt grau statt der Straßenkarte. Beides hat
unterschiedliche Ursachen. Fehlen nur die Symbole, ist am wahrscheinlichsten, dass dein
Zuständigkeitsbereich im dargestellten Kartenausschnitt schlicht noch keine Bäume oder
Gruppen enthält, oder dass ein gesetzter Filter die Anzeige einschränkt; prüfe die
Sucheingabe und den Filter nach Bewässerungszustand oberhalb der Karte, siehe
[Die Karte lesen](./map.md#die-karte-lesen).

Bleibt dagegen der Kartenhintergrund selbst leer, lädt Green Ecolution die Kartenkacheln
von einem externen Kartendienst im Internet, getrennt vom eigenen Server der Anwendung;
ist dieser Dienst über deine Verbindung nicht erreichbar, fehlt der Hintergrund, obwohl
Green Ecolution selbst erreichbar bleibt. Eine fehlende Berechtigung äußert sich dagegen
nicht als leere Karte, sondern als eigene Seite **Kein Zugriff**, siehe
[Eine Seite meldet „Kein Zugriff"](./troubleshooting.md#eine-seite-meldet-kein-zugriff) weiter
oben.

## Die Anwendung meldet eine neue Version

Während der Arbeit öffnet sich ein Dialog mit dem Titel **Neue Version verfügbar** und
den Schaltflächen **Später** und **Jetzt aktualisieren**. Green Ecolution prüft
stündlich im Hintergrund, ob eine neue Version vorliegt, unabhängig davon, ob du sie als
installierte App oder im Browser geöffnet hast, und zeigt diesen Dialog, sobald sie eine
gefunden hat. **Jetzt aktualisieren** lädt die neue
Version und startet die Anwendung damit neu; ist gerade eine Eingabe in Arbeit, sichere
sie vorher oder wähle **Später**, um erst nach Abschluss zu aktualisieren. Der Dialog
kehrt beim nächsten stündlichen Prüflauf zurück, solange die neue Version nicht
eingespielt ist.

Unabhängig davon zeigt der Reiter **Software** auf der Seite
[Systeminformationen](./system-info.md#der-aktualisierungshinweis) denselben Hinweis
dauerhaft an, solange eine neuere Version vorliegt, zusammen mit deren Nummer und einem
Link zu den Release Notes; steht dort **Software ist aktuell**, läuft bereits die
neueste Version. Diese Prüfung läuft nur für regulär veröffentlichte Versionen; auf
einem lokalen Entwicklungs-Build oder einer Staging-Version zeigt die Seite
stattdessen einen festen Hinweis, dass die Prüfung hier bewusst abgeschaltet ist.
