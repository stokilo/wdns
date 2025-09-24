# WDNS Proxy Scripts - Podsumowanie

Ten dokument opisuje wszystkie skrypty do zarządzania proxy WDNS na macOS.

## 🚀 Szybki Start

```bash
# 1. Uruchom wszystko jednym poleceniem
./scripts/quick-start.sh

# 2. Uruchom aplikacje z proxy
./scripts/start-with-proxy.sh all

# 3. Sprawdź status
./scripts/proxy-manager.sh status
```

## 📁 Dostępne Skrypty

### 🎯 **Główne Skrypty**

| Skrypt | Opis | Użycie |
|--------|------|--------|
| **`quick-start.sh`** | 🚀 **Szybki start** | `./scripts/quick-start.sh` |
| **`proxy-manager.sh`** | 🔧 **Menedżer proxy** | `./scripts/proxy-manager.sh [start\|stop\|status\|enable\|disable\|test\|dns\|apps]` |
| **`start-with-proxy.sh`** | 📱 **Uruchamianie aplikacji** | `./scripts/start-with-proxy.sh [terminal\|chrome\|firefox\|safari\|vscode\|all]` |
| **`setup-proxy-env.sh`** | ⚙️ **Konfiguracja środowiska** | `./scripts/setup-proxy-env.sh [enable\|test\|unset]` |

### 🔧 **Zaawansowane Skrypty**

| Skrypt | Opis | Użycie |
|--------|------|--------|
| **`macos-quick-proxy.sh`** | ⚡ **Szybka konfiguracja** | `./scripts/macos-quick-proxy.sh -e` |
| **`macos-proxy-setup.sh`** | 🔧 **Pełna konfiguracja** | `./scripts/macos-proxy-setup.sh` |
| **`proxy`** | 🎯 **Prosty menedżer** | `./scripts/proxy [on\|off\|test\|apps\|status]` |

### 🧪 **Testowe Skrypty**

| Skrypt | Opis | Użycie |
|--------|------|--------|
| **`test-proxy.sh`** | 🧪 **Test proxy** | `./scripts/test-proxy.sh` |
| **`demo-macos-proxy.sh`** | 🎬 **Demo** | `./scripts/demo-macos-proxy.sh` |

## 🎯 **Przykłady Użycia**

### **Codzienne Użycie**

```bash
# 1. Uruchom serwis i proxy
./scripts/quick-start.sh

# 2. Uruchom aplikacje
./scripts/start-with-proxy.sh all

# 3. Pracuj normalnie - wszystko przez proxy

# 4. Zatrzymaj gdy skończysz
./scripts/proxy-manager.sh stop
```

### **Zarządzanie Serwisem**

```bash
# Sprawdź status
./scripts/proxy-manager.sh status

# Uruchom serwis
./scripts/proxy-manager.sh start

# Zatrzymaj serwis
./scripts/proxy-manager.sh stop

# Testuj proxy
./scripts/proxy-manager.sh test

# Pokaż DNS resolution
./scripts/proxy-manager.sh dns
```

### **Konfiguracja Proxy**

```bash
# Włącz proxy (zmienne środowiskowe)
./scripts/proxy-manager.sh enable

# Wyłącz proxy
./scripts/proxy-manager.sh disable

# Testuj proxy
./scripts/proxy-manager.sh test
```

### **Uruchamianie Aplikacji**

```bash
# Uruchom Terminal z proxy
./scripts/start-with-proxy.sh terminal

# Uruchom Chrome z proxy
./scripts/start-with-proxy.sh chrome

# Uruchom Firefox z proxy
./scripts/start-with-proxy.sh firefox

# Uruchom wszystkie aplikacje
./scripts/start-with-proxy.sh all
```

## 🔧 **Konfiguracja Zaawansowana**

### **Szybka Konfiguracja**

```bash
# Włącz proxy bez sudo
./scripts/setup-proxy-env.sh -e

# Testuj konfigurację
./scripts/setup-proxy-env.sh -t

# Wyłącz proxy
./scripts/setup-proxy-env.sh -u
```

### **Pełna Konfiguracja**

```bash
# Pełna konfiguracja systemu (wymaga sudo)
./scripts/macos-proxy-setup.sh

# Szybka konfiguracja
./scripts/macos-quick-proxy.sh -e
```

### **Prosty Menedżer**

```bash
# Włącz proxy
./scripts/proxy on

# Testuj proxy
./scripts/proxy test

# Uruchom aplikacje
./scripts/proxy apps

# Sprawdź status
./scripts/proxy status

# Wyłącz proxy
./scripts/proxy off
```

## 🧪 **Testowanie**

### **Test Proxy**

```bash
# Test podstawowy
./scripts/test-proxy.sh

# Test przez menedżer
./scripts/proxy-manager.sh test

# Demo interaktywne
./scripts/demo-macos-proxy.sh
```

### **Test DNS**

```bash
# Test DNS resolution
./scripts/proxy-manager.sh dns

# Test bezpośredni
curl -X POST http://127.0.0.1:9700/api/dns/resolve \
  -H "Content-Type: application/json" \
  -d '{"hosts": ["google.com", "github.com"]}'
```

## 🚨 **Rozwiązywanie Problemów**

### **Serwis Nie Działa**

```bash
# Sprawdź status
./scripts/proxy-manager.sh status

# Uruchom serwis
./scripts/proxy-manager.sh start

# Sprawdź logi
cat wdns-service.log
```

### **Proxy Nie Działa**

```bash
# Testuj proxy
./scripts/proxy-manager.sh test

# Sprawdź zmienne środowiskowe
env | grep -i proxy

# Włącz proxy
./scripts/proxy-manager.sh enable
```

### **Aplikacje Nie Używają Proxy**

```bash
# Uruchom aplikacje z proxy
./scripts/start-with-proxy.sh all

# Sprawdź konfigurację
./scripts/setup-proxy-env.sh -t
```

## 📋 **Workflow**

### **1. Pierwsze Uruchomienie**

```bash
# Uruchom wszystko
./scripts/quick-start.sh

# Uruchom aplikacje
./scripts/start-with-proxy.sh all
```

### **2. Codzienne Użycie**

```bash
# Sprawdź status
./scripts/proxy-manager.sh status

# Włącz proxy jeśli potrzebne
./scripts/proxy-manager.sh enable

# Uruchom aplikacje
./scripts/start-with-proxy.sh all
```

### **3. Zakończenie Pracy**

```bash
# Wyłącz proxy
./scripts/proxy-manager.sh disable

# Zatrzymaj serwis
./scripts/proxy-manager.sh stop
```

## 🔧 **Konfiguracja**

### **Zmienne Środowiskowe**

```bash
# Automatyczne ustawienie
./scripts/setup-proxy-env.sh -e

# Ręczne ustawienie
export HTTP_PROXY=http://127.0.0.1:9701
export HTTPS_PROXY=http://127.0.0.1:9701
export http_proxy=http://127.0.0.1:9701
export https_proxy=http://127.0.0.1:9701
```

### **Konfiguracja Aplikacji**

```bash
# Chrome
open -a "Google Chrome" --args --proxy-server="http://127.0.0.1:9701"

# Firefox
./scripts/start-with-proxy.sh firefox

# Terminal
./scripts/start-with-proxy.sh terminal
```

## 📚 **Dokumentacja**

- **`README-macos-proxy.md`** - Szczegółowa dokumentacja macOS proxy
- **`README.md`** - Główna dokumentacja WDNS
- **`TESTING.md`** - Dokumentacja testów

## 🎯 **Najczęściej Używane Komendy**

```bash
# Szybki start
./scripts/quick-start.sh

# Sprawdź status
./scripts/proxy-manager.sh status

# Testuj proxy
./scripts/proxy-manager.sh test

# Uruchom aplikacje
./scripts/start-with-proxy.sh all

# Zatrzymaj wszystko
./scripts/proxy-manager.sh stop
```

## 🚀 **Gotowe do Użycia!**

Wszystkie skrypty są gotowe do użycia. Zacznij od:

```bash
./scripts/quick-start.sh
```

I ciesz się pełną funkcjonalnością proxy WDNS na macOS!
