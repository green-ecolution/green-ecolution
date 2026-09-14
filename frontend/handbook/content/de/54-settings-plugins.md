---
slug: settings-plugins
title: Plugins
part: administration
summary: Was ein Plugin ist, wie ein Admin eines installiert, was die beiden Rechtemengen bedeuten, wie der einmalig sichtbare Schlüssel rotiert wird und wie sich die eigene Ansicht eines Plugins öffnen lässt.
routes: ['/settings/plugin', '/settings/plugin/$slug', '/plugin/$slug']
---

> [!NOTE]
> Diese Seite siehst du nur mit der Berechtigung `plugin:read`, und nur, wenn deine
> Instanz die Plugin-Funktion überhaupt eingeschaltet hat. Fehlt dir die Berechtigung
> oder ist die Funktion ausgeschaltet, taucht **Plugins** in deiner Einstellungsnavigation
> gar nicht erst auf. Zum Installieren, Ändern und Deinstallieren brauchst du zusätzlich
> `plugin:create`, `plugin:update` beziehungsweise `plugin:delete`, jeweils in der
> Organisation, der das Plugin gehört.

## Was ein Plugin ist

Ein Plugin ist ein externes System, das Green Ecolution um Daten oder eine eigene
Ansicht erweitert, ohne selbst Teil der Anwendung zu sein. Praktisch bedeutet das: Ein
Plugin bekommt beim Installieren einen eigenen API-Schlüssel und kann darüber Bäume
anlegen, aktualisieren oder löschen, etwa um ein kommunales Baumkataster laufend
einzuspielen. Zusätzlich kann ein Plugin eine eigene Oberfläche mitbringen, die in
Green Ecolution eingebettet erscheint, muss das aber nicht. Ein Plugin registriert sich
dabei nicht selbst; es wird von einer Person mit den passenden Rechten in den
Einstellungen angelegt, ganz wie ein Fahrzeug oder eine Rolle.

Die Übersichtsseite **Plugins** listet jedes installierte Plugin mit Status, Name samt
Slug, Organisation und dem Zeitpunkt seines letzten Kontakts. Dieser Zeitpunkt ist keine
manuelle Angabe, sondern aktualisiert sich mit jeder Anfrage, die das Plugin an seine
Schnittstelle stellt, und zeigt dir damit auf einen Blick, ob ein Plugin überhaupt noch
aktiv angebunden ist.

## Ein Plugin installieren

Über **Plugin installieren** öffnest du den Installationsdialog. Slug, Name und
optional eine Beschreibung sowie die Organisation, der das Plugin gehören soll, sind
Pflichtangaben. Der Slug besteht aus Kleinbuchstaben, Ziffern und Bindestrichen und
lässt sich nach dem Anlegen nicht mehr ändern: Er dient zugleich als Herkunftskennung
aller Daten, die das Plugin importiert, und ein nachträglicher Wechsel würde diese
Zuordnung verwaisen lassen. Einzelne Wörter sind für die Plugin-Schnittstelle selbst
reserviert, derzeit `me`; der Dialog weist sie mit einem Hinweis zurück.

Darunter legst du fest, ob und wie das Plugin eine eigene Ansicht mitbringt: **Keine
Ansicht** für ein reines Datenplugin oder **Extern gehostet** für eine öffentlich
erreichbare Adresse. Eine extern gehostete Ansicht muss eine absolute https-Adresse
sein, mit Ausnahme von localhost für die lokale Entwicklung, und darf nicht auf die
Adresse von Green Ecolution selbst zeigen. Ein dritter Modus für Dienste, die nur
intern laufen und von Green Ecolution durchgereicht werden, ist vorbereitet, aber
noch nicht auswählbar, solange die Durchreichung fehlt.

Zum Schluss legst du die beiden Rechtemengen fest, siehe
[Die beiden Rechtemengen](./settings-plugins.md#die-beiden-rechtemengen) weiter unten.
Nach **Installieren** zeigt ein eigener Dialog den frisch erzeugten API-Schlüssel im
Klartext. Das ist der einzige Moment, in dem du ihn zu sehen bekommst: Green Ecolution
speichert nur einen Hashwert davon und kann ihn dir später nicht erneut anzeigen.
Nimm ihn deshalb gleich über **Schlüssel kopieren** mit und hinterlege ihn dort, wo
das Plugin ihn zur Anmeldung braucht. Ein Klick neben den Dialog schließt ihn
bewusst nicht, damit dir der Schlüssel nicht aus Versehen verloren geht.

Ein frisch installiertes Plugin ist zunächst deaktiviert. Solange es das ist, weist
Green Ecolution seine Anfragen auch mit gültigem Schlüssel ab. Aktiviere es deshalb
auf seiner Detailseite, bevor du den Schlüssel weitergibst, sonst scheitert der erste
Anmeldeversuch des Plugins ohne erkennbaren Grund.

![Der Installationsdialog eines Plugins mit beiden Rechtematrizen](../images/plugin-install.png)

Ein installiertes Plugin lässt sich über seine Detailseite jederzeit umbenennen,
beschreiben, in seiner Ansicht und seinen Rechten anpassen, vorübergehend deaktivieren
oder ganz deinstallieren. Deaktivieren sperrt lediglich die Anmeldung des Plugins an
seiner Schnittstelle, ohne bereits importierte Daten anzutasten; erst **Deinstallieren**
entfernt das Plugin selbst.

> [!WARNING]
> Beim Deinstallieren bleiben die vom Plugin angelegten Bäume zwar erhalten, ihre
> Verknüpfung mit dem Plugin geht dabei aber verloren. Installierst du dasselbe Plugin
> später erneut, bekommt es eine neue Kennung und erkennt die alten Datensätze nicht
> wieder: Der nächste Import legt sie allesamt ein zweites Mal an, und an die
> ursprünglichen Datensätze kommt das Plugin nicht mehr heran. Bei einem Kataster mit
> zehntausenden Bäumen ist das eine große Menge an Dubletten. Zum vorübergehenden
> Stilllegen eines Plugins ist **Deaktivieren** deshalb der richtige Weg.

## Die beiden Rechtemengen

Der Installationsdialog und die Detailseite zeigen zwei getrennte Rechtematrizen, die
auf den ersten Blick ähnlich aussehen, aber unterschiedliche Fragen beantworten.

**Rechte des Plugins** legt fest, was das Plugin selbst tun darf, wenn es Daten in
Green Ecolution schreibt, etwa Bäume anlegen oder bearbeiten. Diese Rechte darfst du
dem Plugin nur bis zu deinen eigenen Rechten geben; verlangt der Dialog mehr, als du
selbst besitzt, weist Green Ecolution die Eingabe zurück.

**Zugriffsrechte für die Ansicht** legt fest, wer in deiner Organisation die eigene
Oberfläche des Plugins überhaupt öffnen darf, falls es eine hat. Diese Menge übernimmt
beim Anlegen zunächst die Rechte des Plugins, ist davon aber unabhängig: Änderst du sie
eigenständig, bleibt sie ab diesem Zeitpunkt getrennt von den Rechten des Plugins
bestehen. Der Grund für die Trennung ist, dass eine Ansicht mit eigenen
Aktionsknöpfen die Rechte des Plugins an die Person weiterreicht, die sie bedient; wer
also die Zugriffsrechte lockert, sollte bewusst tun, wessen Handlungen er damit
zulässt.

Wer die Ansicht öffnen will, muss alle hier gewählten Rechte besitzen, und zwar in der
Organisation des Plugins oder darüber. Ein einzelnes davon genügt nicht. Umgekehrt ist
`plugin:read` dafür nicht nötig: Diese Berechtigung verwaltet Plugins, sie bedient sie
nicht. Genau darum können Kolleginnen und Kollegen ohne jede Plugin-Berechtigung die
Ansicht nutzen, sobald ihre Rolle die hier gewählten Rechte enthält.

## Den API-Schlüssel rotieren

Vermutest du, dass der Schlüssel eines Plugins bekannt geworden ist, oder wechselt
das Plugin den Betreiber, erneuerst du ihn über **Schlüssel erneuern** auf der
Detailseite. Nach Bestätigung wird der bisherige Schlüssel sofort ungültig, und ein
neuer erscheint, wie beim Installieren, genau einmal im Klartext im selben Dialog.
Bis das Plugin mit dem neuen Schlüssel hinterlegt ist, kann es sich in dieser Zeit
nicht anmelden und keine Daten mehr schreiben.

Erneuern und Aktivieren verlangen dieselbe Bedingung wie das Installieren: Du musst
alle Rechte, die das Plugin selbst besitzt, auch selbst haben. Ein neuer Schlüssel
ist ein funktionierender Zugang zu allem, was das Plugin darf, und das Aktivieren
setzt genau diese Rechte in Betrieb. Umbenennen, Beschreiben und Deaktivieren
verlangen das nicht, damit niemand daran gehindert wird, ein Plugin abzuschalten.

## Die Ansicht eines Plugins öffnen

Bringt ein Plugin eine externe Ansicht mit, öffnest du sie über **Ansicht öffnen** auf
seiner Detailseite. Green Ecolution bettet sie in einem eigenen, abgeschotteten Rahmen
ein und übergibt ihr deinen Anzeigenamen, die Oberflächensprache und den Slug des
Plugins, aber keinen Zugriff auf deine Anmeldedaten. Ein Plugin ohne eigene Ansicht
zeigt an dieser Stelle stattdessen einen Hinweis, dass es keine besitzt.

Die Detailseite ist allerdings nur der Weg der Administration. Alle anderen erreichen
die Ansicht über ihre eigene Adresse `/plugin/<slug>`, also etwa
`https://deine-instanz.example/plugin/demo-plugin`. Einen Eintrag in der Navigation gibt
es dafür bislang nicht; gib den Link deshalb nach dem Installieren an die Personen
weiter, die mit dem Plugin arbeiten sollen. Fehlt jemandem eines der Zugriffsrechte,
erscheint die übliche Meldung über den fehlenden Zugriff, und dasselbe passiert, solange
das Plugin deaktiviert ist.
