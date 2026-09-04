---
slug: system-info
title: Systeminformationen
part: administration
summary: Was die Systeminformationen zeigen, wozu sie bei einer Störungsmeldung nützlich sind und was der Aktualisierungshinweis bedeutet.
routes: ['/info']
---

Diese Seite hat noch keinen eigenen Eintrag in der Seitennavigation; du erreichst sie
direkt über die Adresse `/info`. Angemeldet sein musst du dafür, eine bestimmte
Berechtigung braucht es aber nicht, die Seite steht jeder angemeldeten Person offen.

## Was die Systeminformationen zeigen

Die Seite gliedert sich in bis zu vier Reiter. **System** zeigt die laufende
Softwareversion mit dem Hinweis, ob sie aktuell ist, die Laufzeit seit dem letzten
Neustart des Servers sowie den Zustand der angebundenen Dienste: Datenbank,
Authentifizierung, MQTT, Routing und das Plugin-System, jeweils mit Namen,
Verbindungsstatus und Antwortzeit. Ist ein Dienst für diese Instanz bewusst
ausgeschaltet, zeigt er **Deaktiviert** statt eines Fehlers.

**Daten** zählt, wie viele Bäume, Bewässerungsgruppen, Sensoren, Fahrzeuge und
Einsatzpläne insgesamt verwaltet werden, als Diagramm und als einzelne Kacheln mit
Link auf die jeweilige Übersicht; die Zahlen gelten für die ganze Instanz, nicht nur
für deine eigene Organisation. **Software** listet Versionsnummer, Build-Datum und
-Zeit, die verwendete Rust-Version sowie Branch, Commit und Repository des Builds. Ein
vierter Reiter, **Server**, erscheint zusätzlich, sobald das Backend Angaben zum Host
liefert: Hostname, Betriebssystem und Architektur, HTTP-Port, Netzwerk-Interface und
die Adresse, unter der der Server erreichbar ist.

## Bei einer Störungsmeldung

Meldest du der Verwaltung oder dem Support eine Störung, liefert diese Seite die
Angaben, die dafür gebraucht werden: die genaue Versionsnummer mit Commit aus dem
Reiter **Software**, welcher der angebundenen Dienste im Reiter **System** gerade
keine Verbindung meldet und mit welcher Statusmeldung, sowie, falls vorhanden,
Hostname und Port aus dem Reiter **Server**. Damit lässt sich eine Störung einer
bestimmten Instanz und einem bestimmten Softwarestand zuordnen, statt nur „es
funktioniert nicht“ zu melden.

## Der Aktualisierungshinweis

Nur bei einer regulären, veröffentlichten Version prüft die Anwendung in
regelmäßigen Abständen, ob es beim Projekt eine neuere Version gibt, und zeigt im
Reiter **Software** entweder **Software ist aktuell** oder **Neue Version verfügbar**
mit der Nummer der neuesten Version und einem Link zu ihren Release Notes. Läuft
stattdessen ein lokaler Entwicklungs-Build oder eine Staging-Version, zeigt die Seite
stattdessen einen festen Hinweis, dass diese Prüfung für diesen Softwarestand bewusst
abgeschaltet ist, weil er ohnehin nicht die produktiv aktualisierte Version ist.
