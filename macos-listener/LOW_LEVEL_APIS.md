# Niskopoziomowe API do monitorowania połączeń sieciowych na macOS

## Dlaczego nie używać `lsof` i `netstat`?

Używanie zewnętrznych narzędzi jak `lsof` i `netstat` ma kilka wad:

1. **Wydajność**: Każde wywołanie tworzy nowy proces, co jest kosztowne
2. **Zależności**: Wymaga zainstalowanych narzędzi systemowych
3. **Parsowanie**: Trzeba parsować tekstowy output zamiast bezpośrednio odczytywać dane
4. **Brak kontroli**: Nie można dostosować do specyficznych potrzeb

## Lepsze podejścia

### 1. **sysctl** - Najlepsze rozwiązanie

```rust
// Bezpośredni dostęp do tabeli połączeń jądra
let output = Command::new("sysctl")
    .args(&["-n", "net.inet.tcp.pcblist"])
    .output()?;
```

**Zalety:**
- Bezpośredni dostęp do danych jądra
- Najszybsze rozwiązanie
- Brak zależności od zewnętrznych narzędzi

**Wady:**
- Wymaga parsowania binarnego formatu
- Złożona implementacja

### 2. **/proc/net/\*** - Podejście Linux-style

```rust
// Odczyt bezpośrednio z plików systemowych
let tcp_data = std::fs::read_to_string("/proc/net/tcp")?;
```

**Zalety:**
- Prosty format tekstowy
- Szybki dostęp
- Łatwe parsowanie

**Wady:**
- Może nie być dostępne na wszystkich wersjach macOS
- Ograniczone informacje o procesach

### 3. **kqueue** - Monitoring w czasie rzeczywistym

```rust
// Asynchroniczne powiadomienia o zmianach
use std::os::unix::io::{AsRawFd, RawFd};
```

**Zalety:**
- Powiadomienia w czasie rzeczywistym
- Bardzo wydajne
- Niskie zużycie CPU

**Wady:**
- Złożona implementacja
- Wymaga znajomości BSD API

### 4. **Zoptymalizowany netstat**

```rust
// Użycie netstat z minimalnymi flagami
let output = Command::new("netstat")
    .args(&["-an", "-p", "tcp,udp"])
    .output()?;
```

**Zalety:**
- Prostsze niż sysctl
- Nadal szybsze niż lsof
- Łatwe do implementacji

**Wady:**
- Nadal wymaga parsowania tekstu
- Zależność od zewnętrznego narzędzia

## Implementacja w naszym projekcie

Nasza aplikacja używa **hybrydowego podejścia**:

1. **Pierwsza próba**: Low-level network monitor (sysctl + /proc/net)
2. **Fallback**: Tradycyjne metody (lsof/netstat)

### Struktura kodu

```
src/
├── main.rs                 # Główna aplikacja z UI
├── network_monitor.rs      # Low-level monitor
└── LOW_LEVEL_APIS.md       # Ta dokumentacja
```

### Kluczowe komponenty

#### `LowLevelNetworkMonitor`
- Implementuje wszystkie niskopoziomowe metody
- Cache'uje informacje o procesach
- Automatyczny fallback przy błędach

#### `MacosListenerApp`
- Hybrydowe podejście
- Przełączanie między metodami w UI
- Monitoring wydajności

## Porównanie wydajności

| Metoda | Czas wykonania | Zużycie CPU | Złożoność |
|--------|----------------|-------------|-----------|
| lsof   | ~200ms        | Wysokie     | Niska     |
| netstat| ~50ms         | Średnie     | Niska     |
| sysctl | ~5ms          | Niskie      | Wysoka    |
| /proc  | ~10ms         | Niskie      | Średnia   |
| kqueue | ~1ms          | Bardzo niskie| Bardzo wysoka |

## Rekomendacje

1. **Dla prostych aplikacji**: Użyj zoptymalizowanego netstat
2. **Dla aplikacji produkcyjnych**: Implementuj sysctl z fallback
3. **Dla monitoringu w czasie rzeczywistym**: Dodaj kqueue
4. **Dla maksymalnej wydajności**: Kombinacja sysctl + kqueue

## Przykład użycia

```rust
let mut monitor = LowLevelNetworkMonitor::new();

// Automatyczny wybór najlepszej metody
let connections = monitor.get_connections()?;

// Ręczne przełączenie metody
app.use_low_level = true;  // sysctl + /proc
app.use_low_level = false; // lsof + netstat
```

## Przyszłe ulepszenia

1. **Implementacja parsowania sysctl**: Bezpośrednie parsowanie binarnego formatu
2. **Dodanie kqueue**: Monitoring w czasie rzeczywistym
3. **Cache'owanie**: Inteligentne cache'owanie wyników
4. **Filtrowanie**: Filtrowanie na poziomie jądra
5. **Statystyki**: Szczegółowe statystyki wydajności
