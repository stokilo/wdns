# UI Buttons Guide - Test Hostname Resolution

## 🎯 **Problem Solved**

Dodałem przycisk **"🧪 Test Hostname"** w głównym panelu UI, żeby był bardziej widoczny i łatwiejszy do znalezienia.

## 📍 **Lokalizacja Przycisku**

### **Główny Panel (Top Panel)**
```
🔍 macOS Network Connection Monitor
Total: X | TCP: X | UDP: X | Listening: X | Established: X
─────────────────────────────────────────────────────────────
Update interval: [slider] | Local only | Remote only | Low-level API
─────────────────────────────────────────────────────────────
Proxy: [Enable Proxy Routing] | Configure Proxies | Manage Rules | 🧪 Test Hostname
```

### **Sekcja Traffic Interceptor**
```
Real Proxy (Actual Traffic Routing): [Enable Real Traffic Proxy]
─────────────────────────────────────────────────────────────
[Start Traffic Interception] [Stop Traffic Interception]
[View Intercepted Traffic]
[Start Traffic Interceptor] [Stop Traffic Interceptor]
```

## 🚀 **Jak Użyć**

### **Krok 1: Uruchom Aplikację**
```bash
cd macos-listener
cargo run
```

### **Krok 2: Znajdź Przycisk**
- Szukaj przycisku **"🧪 Test Hostname"** w głównym panelu
- Przycisk jest obok "Configure Proxies" i "Manage Rules"
- Ma emoji 🧪 żeby był bardziej widoczny

### **Krok 3: Otwórz Dialog Testowy**
1. Kliknij **"🧪 Test Hostname"**
2. Otworzy się dialog "Test Hostname Resolution"
3. Wpisz hostname: `networkpartner-kion-dev.cprt.kion.cloud`
4. Testuj różne metody rozwiązywania

## 🧪 **Testy Dostępne**

### **1. Test Direct DNS**
- Testuje standardowe rozwiązywanie DNS
- **Oczekiwany wynik**: ❌ **FAILED** (hostname nie jest publicznie rozwiązywalny)

### **2. Test via SOCKS5**
- Testuje przez SOCKS5 proxy (192.168.0.115:9702)
- **Oczekiwany wynik**: ✅ **SUCCESS** (proxy rozwiązuje hostname zdalnie)

### **3. Test via Interceptor**
- Testuje przez traffic interceptor (127.0.0.1:5353)
- **Oczekiwany wynik**: ✅ **SUCCESS** (jeśli interceptor jest skonfigurowany)

## 📊 **Oczekiwane Wyniki**

### **Dla `networkpartner-kion-dev.cprt.kion.cloud`:**

#### **Direct DNS Test** ❌
```
❌ Direct DNS Failed:
nslookup: can't resolve 'networkpartner-kion-dev.cprt.kion.cloud'
```

#### **SOCKS5 Proxy Test** ✅
```
✅ SOCKS5 Proxy Test:
* Trying 192.168.0.115:9702...
* Connected to 192.168.0.115 (192.168.0.115) port 9702
* SOCKS5 connect to networkpartner-kion-dev.cprt.kion.cloud:80 (remotely resolved)
* HTTP/1.1 200 OK
```

#### **Interceptor Test** ✅
```
✅ DNS via Interceptor:
Name: networkpartner-kion-dev.cprt.kion.cloud
Address: [resolved IP address]

✅ HTTP via Interceptor:
* Connected to [resolved IP] port 80
* HTTP/1.1 200 OK
```

## 🔧 **Konfiguracja Wymagana**

### **1. SOCKS5 Proxy**
- **Host**: 192.168.0.115
- **Port**: 9702
- **Typ**: SOCKS5
- **Uwierzytelnianie**: Jeśli wymagane

### **2. Reguły Traffic Interceptor**
- **Wzorzec**: `*.kion.cloud` lub `*.cprt.kion.cloud`
- **Proxy**: Przypisz do SOCKS5 proxy
- **Status**: Włączony

### **3. Status Traffic Interceptor**
- **DNS Interceptor**: Działa na porcie 5353
- **Monitorowanie TCP/UDP**: Aktywne
- **Dopasowywanie reguł**: Włączone

## 🐛 **Rozwiązywanie Problemów**

### **Jeśli nie widzisz przycisku:**
1. **Sprawdź rozmiar okna** - może być za małe
2. **Przewiń w dół** - przycisk może być poza widokiem
3. **Sprawdź główny panel** - przycisk jest obok "Configure Proxies"

### **Jeśli przycisk nie działa:**
1. **Sprawdź kompilację**: `cargo check`
2. **Uruchom ponownie**: `cargo run`
3. **Sprawdź logi** w terminalu

### **Jeśli testy nie działają:**
1. **Sprawdź połączenie** z SOCKS5 proxy
2. **Sprawdź konfigurację** reguł
3. **Sprawdź status** traffic interceptor

## 📝 **Skrypty Testowe**

### **Test UI Buttons**
```bash
./test_ui_buttons.sh
```

### **Test Hostname Resolution**
```bash
./test_hostname_resolution.sh
```

### **Test Traffic Interceptor**
```bash
./test_traffic_interceptor.sh
```

## 🎯 **Następne Kroki**

1. **Uruchom aplikację**: `cargo run`
2. **Znajdź przycisk**: "🧪 Test Hostname" w głównym panelu
3. **Otwórz dialog**: Kliknij przycisk
4. **Wpisz hostname**: `networkpartner-kion-dev.cprt.kion.cloud`
5. **Testuj metody**: Sprawdź wszystkie trzy metody
6. **Porównaj wyniki**: Zobacz która metoda działa
7. **Skonfiguruj interceptor**: Jeśli potrzebne

## ✅ **Podsumowanie**

Przycisk **"🧪 Test Hostname"** jest teraz widoczny w głównym panelu UI obok innych przycisków konfiguracyjnych. Użyj go do testowania rozwiązywania hostname i weryfikacji, czy traffic interceptor działa poprawnie.

**Lokalizacja**: Główny panel → "🧪 Test Hostname" (obok "Configure Proxies")
**Funkcja**: Testowanie rozwiązywania hostname przez różne metody
**Cel**: Weryfikacja czy traffic interceptor przekierowuje ruch przez SOCKS5 proxy
