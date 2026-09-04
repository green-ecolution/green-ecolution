---
slug: watering-plans
title: Einsatzpläne
part: planning
summary: Das Board verstehen, einen Einsatz anlegen und seine Route festlegen, und wie ein Einsatzplan seinen Zustandsablauf bis zur Auswertung durchläuft.
routes:
  [
    '/watering-plans',
    '/watering-plans/$wateringPlanId',
    '/watering-plans/new',
    '/map/watering-plan/select/cluster',
  ]
---

## Das Board verstehen

Unter **Einsatzpläne** in der Seitennavigation öffnet sich ein Board mit vier Spalten.
Ganz links liegen **Vorschläge**: Bewässerungsgruppen, deren Zustand aktuell
**Sehr trocken** ist und die noch keinem Einsatzplan zugeordnet sind. Markierst du dort
eine oder mehrere Gruppen und klickst auf **Zu Einsatzplan bündeln**, öffnet sich das
Formular für einen neuen Einsatzplan mit diesen Gruppen bereits vorausgewählt; auswählen
und bündeln kann nur, wer neue Einsatzpläne anlegen darf.

Die drei übrigen Spalten fassen die eigentlichen Einsatzpläne nach Zustand zusammen:
**Geplant**, **Unterwegs** und **Erledigt**. Unterwegs zeigt Einsatzpläne im Zustand
Aktiv; Erledigt fasst die drei abgeschlossenen Zustände Beendet, Abgebrochen und Nicht
angetreten in einer Spalte zusammen. Jede Karte nennt Datum, Anzahl der Bewässerungsgruppen,
benötigte Wassermenge sowie das oder die eingesetzten Fahrzeuge; ein Klick auf eine Karte
öffnet den vollständigen Einsatzplan mit seinen Reitern **Allgemeine Daten** und
**Bewässerungsgruppen**, dem Download der Route und, sobald eine Route berechnet ist,
einem zusätzlichen Reiter **Route**. Wer Einsatzpläne ändern darf, kann Karten aus der
Spalte Geplant zudem per Ziehen in eine andere Spalte fallen lassen, um ihren Zustand zu
wechseln; dazu gleich mehr.

Jeder Einsatzplan durchläuft dabei einen festen Zustandsablauf:

| Zustand          | Bedeutung                                                       |
| ---------------- | --------------------------------------------------------------- |
| Geplant          | Der Einsatzplan ist geplant und kann gestartet werden.          |
| Aktiv            | Der Einsatzplan ist aktiv und wird aktuell ausgeführt.          |
| Beendet          | Der Einsatzplan wurde erfolgreich beendet.                      |
| Abgebrochen      | Der Einsatzplan wurde abgebrochen und ist nicht fertiggestellt. |
| Nicht angetreten | Der Einsatzplan wurde nicht angetreten.                         |
| Unbekannt        | Der Status des Einsatzplans ist unbekannt.                      |

Ein neu angelegter Einsatzplan startet immer als Geplant. Von dort aus lässt er sich
starten (Aktiv) oder abbrechen (Abgebrochen); ein gestarteter Einsatzplan lässt sich
außerdem als nicht angetreten melden (Nicht angetreten) oder mit einer Auswertung
beenden (Beendet), und ein versehentlicher Start lässt sich zurücknehmen. Beendet,
Abgebrochen und Nicht angetreten sind Endzustände: Aus ihnen heraus lässt sich weder der
Inhalt noch der Status eines Einsatzplans mehr ändern. Unbekannt ist ein technischer
Übergangswert aus älteren Daten, dem du im laufenden Betrieb nicht begegnest.

## Einsatz anlegen

Auf dem Board legst du einen neuen Einsatzplan über **Neuen Einsatzplan erstellen** an.
Trage das Datum des Einsatzplans, das einzusetzende Fahrzeug (**Verknüpftes Fahrzeug**)
und den Startpunkt ein, von dem aus die Fahrt beginnt und zu dem sie zurückkehrt; optional
trägst du unter **Verknüpfter Anhänger** einen Anhänger ein. Unter **Verknüpfte Mitarbeitende** wählst du
die Personen aus, die den Einsatz durchführen; mit gedrückter Shift-Taste lassen sich
mehrere auf einen Griff markieren. Eine kurze Beschreibung ist optional. Welche
Bewässerungsgruppen angefahren werden, legst du im nächsten Abschnitt fest.

Ein neu erstellter Einsatzplan wird automatisch als Geplant eingestuft.

> [!IMPORTANT]
> Datum, Fahrzeuge, Mitarbeitende und Bewässerungsgruppen eines Einsatzplans lassen sich
> nur ändern, solange er noch **Geplant** ist. Sobald er gestartet wurde, nimmt die
> Anwendung Änderungen an diesen Angaben nicht mehr an; nur sein Status lässt sich dann
> noch anpassen, siehe die folgenden Abschnitte.

Für die zugewiesenen Fahrzeuge muss mindestens eine der ausgewählten Personen die
passende Führerscheinklasse mitbringen, sonst lässt sich der Einsatzplan nicht speichern;
mehr dazu im [Kapitel zu den Fahrzeugen](./vehicles.md#fuhrerscheinklassen).

Auf der Bearbeitungsseite eines bestehenden Einsatzplans findet sich, sofern die
Berechtigung dazu vorliegt, außerdem die Schaltfläche **Löschen**: Sie entfernt den
Einsatzplan unabhängig von seinem Zustand endgültig, samt seiner Kommentare; anders als
beim [Archivieren eines Fahrzeugs](./vehicles.md#fahrzeug-archivieren) lässt sich das
nicht rückgängig machen.

## Route festlegen

Welche Bewässerungsgruppen ein Einsatzplan anfährt, fügst du über
**Bewässerungsgruppen hinzufügen** im Formular hinzu; die Schaltfläche öffnet die Karte
im Auswahlmodus **Bewässerungsgruppen auswählen**. Dort klickst du die gewünschten
Gruppen einzeln an, ein erneuter Klick wählt eine Gruppe wieder ab. Gruppen, deren Bäume
zusammen mehr Wasser benötigen, als das gewählte Fahrzeug und ein eventueller Anhänger
zusammen fassen, sind ausgegraut und lassen sich nicht auswählen; die Anwendung rechnet
dafür mit 80 Litern je Baum einer Gruppe. Ist noch kein Fahrzeug gewählt, weist ein
Hinweis darauf hin, dass ohne Fahrzeug keine Route berechnet werden kann. Sobald
mindestens eine Gruppe und ein Fahrzeug feststehen, zeichnet die Karte zur Orientierung
bereits eine vorläufige Route ein. **Übernehmen** speichert die Auswahl im Formular und
führt zurück zum Einsatzplan.

Die endgültige Route berechnet die Anwendung selbst, sobald du den Einsatzplan
speicherst: aus Startpunkt, gewähltem Fahrzeug samt Anhänger und den ausgewählten Gruppen
ermittelt sie Streckenlänge, benötigte Wassermenge, Anzahl nötiger Nachfüllungen an einem
Wasserdepot und die voraussichtliche Fahrzeit. Solange der Einsatzplan noch Geplant ist,
löst jede inhaltliche Änderung automatisch eine neue Berechnung aus. Konnte keine Route
berechnet werden, weist die Detailseite des Einsatzplans darauf hin und schlägt vor zu
prüfen, ob das gewählte Fahrzeug über ausreichend Wasserkapazität für die gewählten
Bewässerungsgruppen verfügt. Liegt eine Route vor, lädst du sie über
**Route herunterladen** als GPX-Datei herunter und siehst sie zusätzlich im Reiter
**Route** auf einer interaktiven Karte.

## Einsatz starten

Einen geplanten Einsatz startest du auf zwei Wegen: Entweder ziehst du seine Karte auf
dem Board von der Spalte Geplant in die Spalte Unterwegs, oder du öffnest den
Einsatzplan und klickst dort auf **Status aktualisieren**, wählst als neuen Status Aktiv
und speicherst. Nach dem Ziehen bestätigt eine Meldung den Start und bietet für kurze
Zeit **Rückgängig** an, um ihn zurückzunehmen; danach lässt sich ein gestarteter
Einsatzplan über **Status aktualisieren** ebenso wieder auf Geplant zurücksetzen.

## Einsatz abbrechen oder als nicht angetreten melden

Ein Einsatzplan lässt sich in zwei unterschiedlichen Situationen vorzeitig beenden.
Abgebrochen passt sowohl auf einen noch nicht gestarteten als auch auf einen bereits
laufenden Einsatz, etwa wenn sich die Planung noch vor der Fahrt ändert oder die Fahrt
selbst unterbrochen werden muss. Nicht angetreten gilt ausschließlich für einen bereits
gestarteten Einsatz, der gar nicht erst ausgeführt wurde, zum Beispiel bei einem
Fahrzeugausfall. Beide Zustände verlangen eine kurze Begründung.

Zum Abbrechen ziehst du die Karte eines geplanten Einsatzes auf dem Board in die Spalte
Erledigt; die Anwendung fragt dann direkt nach dem **Grund des Abbruchs**. Für einen
bereits laufenden Einsatz bietet dasselbe Ziehen in die Spalte Erledigt die Wahl
zwischen Beendet und Abgebrochen. Alternativ setzt du auf der Detailseite über
**Status aktualisieren** den Status ebenfalls auf Abgebrochen.

> [!NOTE]
> Nicht angetreten lässt sich nur auf der Detailseite über **Status aktualisieren**
> setzen, dort mit einer Begründung im Feld **Grund des Nichtantritts**; per Ziehen auf
> dem Board steht dieser Zustand nicht zur Verfügung.

Ist ein Einsatzplan einmal Beendet, Abgebrochen oder Nicht angetreten, bietet ihm die
Detailseite keine Möglichkeit mehr, seinen Status zu ändern.

## Einsatz mit Auswertung beenden

Einen laufenden Einsatz schließt du ab, indem du seine Karte auf dem Board in die
Spalte Erledigt ziehst und dort Beendet wählst, oder auf der Detailseite über
**Status aktualisieren** den Status auf Beendet setzt. In beiden Fällen trägst du für
jede zugewiesene Bewässerungsgruppe die tatsächlich verbrauchte Wassermenge in Litern
ein; als Ausgangswert schlägt die Anwendung 80 Liter je Baum der Gruppe vor, du kannst
den Wert für jede Gruppe einzeln anpassen. Für jede Gruppe muss ein Wert größer null
hinterlegt sein, ein Einsatzplan lässt sich nur beenden, wenn für jede seiner Gruppen ein
solcher Wert vorliegt.

Sobald der Einsatzplan Beendet ist, zeigt seine Detailseite die insgesamt verbrauchte
Wassermenge, und seine Bewässerungsgruppen wechseln auf **Soeben bewässert**, siehe
[Bewässerungsstatus und wie er zustande kommt](./treecluster.md#bewasserungsstatus-und-wie-er-zustande-kommt).

## Kommentare

Am Fuß der Detailseite können alle, die auf den Einsatzplan zugreifen dürfen, Kommentare
hinterlassen, etwa um Beobachtungen zur Fahrt oder Absprachen zum nächsten Einsatz
festzuhalten. Eigene Kommentare lassen sich nachträglich bearbeiten, was als
**(bearbeitet)** vermerkt wird, oder wieder löschen, wobei die Anwendung vor dem
endgültigen Löschen noch einmal nachfragt; Kommentare anderer lassen sich nur lesen.
