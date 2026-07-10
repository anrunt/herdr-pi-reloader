# TUI — plan implementacji krok po kroku

Ten dokument przekłada wymagania z `plans/tui-plan.md` na bezpieczną ścieżkę nauki i implementacji. `tui-plan.md` pozostaje źródłem prawdy dla zachowania końcowego.

## Jak będziemy pracować

- Realizujemy tylko jeden krok naraz.
- Przed każdym krokiem wyjaśniam nowy koncept i pokazuję mały zakres zmian.
- Po każdym kroku uruchamiamy `cargo fmt` i `cargo check` oraz wykonujemy tylko potrzebną weryfikację ręczną.
- Nie uruchamiamy automatycznie `reload` ani `reset`, ponieważ mają skutki uboczne w działających panelach Pi.
- Nie przechodzimy dalej, dopóki bieżący krok nie kompiluje się i nie jest zrozumiały.
- Nie dodajemy testów, zgodnie z jawnym non-goal planu produktu.

## Co już istnieje i czego brakuje

Obecny projekt ma trzy małe moduły:

- `src/main.rs` — rozpoznaje polecenia `reload` i `reset`;
- `src/herdr.rs` — pobiera agentów i wykonuje komendy w panelach Herdr;
- `src/pi.rs` — wybiera kandydatów i wykonuje reload/reset.

Przed podłączeniem TUI trzeba rozwiązać trzy ważne problemy:

1. Funkcje operacyjne używają `println!`, co uszkodzi ekran kontrolowany przez TUI.
2. Reload ma częściowe podsumowanie, ale reset nie zwraca kompletnego podsumowania wykonania.
3. Deserializacja agentów jest obecnie ścisła. Brak pola może przerwać parsowanie całej listy, podczas gdy wymagania rozróżniają globalny błąd listy od błędu pojedynczego, rozpoznanego agenta Pi.

## Docelowy podział odpowiedzialności

- `main.rs` wybiera tryb: bezpośrednie CLI albo TUI.
- `herdr.rs` komunikuje się z Herdr i zwraca dane/błędy; niczego nie drukuje podczas operacji.
- `pi.rs` klasyfikuje agentów, wykonuje operacje i zwraca strukturalne podsumowanie.
- nowy `tui.rs` posiada stan aplikacji, obsługę klawiatury, renderowanie i cykl życia terminala.
- `herdr-plugin.toml` wyłącznie otwiera zarządzany panel overlay z poleceniem TUI.

Planowany stan aplikacji:

```text
Menu -> Running -> Result -> Exit
                  \-> Error -> Exit
```

Operacja działa niezależnie od odświeżania ekranu. Dzięki temu spinner i resize pozostają responsywne, a klawisze są ignorowane w stanie `Running` bez anulowania pracy.

## Etapy

### Etap 1 — bezpieczny punkt wejścia `tui`

Dodamy osobne polecenie `tui`, ale jeszcze bez Ratatui. Pozwoli to najpierw zrozumieć routing CLI i granicę nowego modułu bez jednoczesnego uczenia się terminala.

### Etap 2 — minimalny ekran terminalowy

Dodamy `ratatui` i `crossterm`, włączymy raw mode i alternate screen, narysujemy statyczny ekran oraz zapewnimy przywrócenie terminala po normalnym wyjściu i błędzie.

### Etap 3 — menu i klawiatura

Wprowadzimy `Menu`, wybór reload/reset, Up/Down, `j`/`k`, zawijanie wyboru, resize oraz klawisze zamykające. Na tym etapie nadal nie wykonujemy prawdziwych operacji.

### Etap 4 — strukturalny wynik reload

Oddzielimy logikę od prezentacji: usuniemy drukowanie z niższych warstw, uodpornimy klasyfikację rekordów agentów i zwrócimy publiczne liczniki oraz błędy zgodne ze specyfikacją. Bezpośrednie CLI nadal wypisze czytelny wynik.

### Etap 5 — kompletna agregacja reset

Zbudujemy `reset_all_pi`, zachowując współbieżność kandydatów i 15-sekundowe oczekiwanie. Funkcja policzy sukcesy, pominięcia, błędy danych, błędy wykonania i błędy zadań Tokio.

### Etap 6 — stan `Running` i spinner

Po Enter pobierzemy świeżą listę agentów, uruchomimy wybraną operację w osobnym zadaniu i będziemy dalej odświeżać ekran. Wszystkie klawisze w tym stanie zostaną świadomie zignorowane.

### Etap 7 — ekrany końcowe

Dodamy `Success`, `Completed with errors`, `Nothing to do` i globalny `Error`, właściwe kolory i tekst niezależny od kolorów, wszystkie liczniki oraz maksymalnie pięć komunikatów błędów z dopiskiem o pozostałych.

### Etap 8 — integracja z Herdr i odbiór całości

Dopiero gdy samo `cargo run --quiet -- tui` działa poprawnie, zmienimy manifest na akcję `open`, dodamy zarządzany panel `overlay` i zaktualizujemy istniejący skrót. Na końcu przejdziemy ręcznie przez checklistę akceptacyjną z `tui-plan.md`.

## Przewidywany wpływ na pliki

- `Cargo.toml`, `Cargo.lock` — Ratatui i Crossterm;
- `src/main.rs` — routing `tui` i zachowanie bezpośrednich komend;
- `src/tui.rs` — nowy moduł UI;
- `src/herdr.rs` — dane odporne na wadliwe rekordy i brak drukowania;
- `src/pi.rs` — wspólne, strukturalne wyniki reload/reset;
- `herdr-plugin.toml` — akcja `open` i panel overlay;
- `~/.config/herdr/config.toml` — podmiana celu istniejącego skrótu.

## Bieżący krok: Etap 1

Zakres pierwszej zmiany:

1. Dodać `src/tui.rs` z prostą funkcją startową wypisującą tymczasowy komunikat.
2. Zadeklarować moduł w `src/main.rs`.
3. Rozszerzyć routing i usage o komendę `tui`, nie zmieniając zachowania `reload` i `reset`.
4. Uruchomić `cargo fmt`, `cargo check` i `cargo run --quiet -- tui`.

Poza zakresem tego kroku: zależności terminalowe, raw mode, renderowanie, obsługa klawiszy, refaktor operacji i manifest Herdr.

Etap 1 jest ukończony, gdy projekt się kompiluje, `tui` uruchamia wyłącznie placeholder, a dotychczasowe gałęzie `reload` i `reset` pozostały funkcjonalnie niezmienione.
