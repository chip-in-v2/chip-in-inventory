#!/bin/sh
# Script to build the 'quench' realm mirroring 'forge' realm using curl

set -e

API_BASE_URL=${API_BASE_URL:-"http://localhost:3000/v1"}
REALM_NAME="quench"
ZONE_NAME="test.v2.chip-in.net"

if ! command -v jq > /dev/null 2>&1; then
    echo "Error: 'jq' is required to run this script."
    exit 1
fi

echo "==> Building realm: $REALM_NAME ($ZONE_NAME)"

# Keys mirrored from 'forge' realm in config.yaml
CACERT="-----BEGIN CERTIFICATE-----
MIIDmTCCAoGgAwIBAgIUQFlH+y76TW4GCwvGWEESfT4sTZAwDQYJKoZIhvcNAQEL
BQAwXDELMAkGA1UEBhMCSlAxDjAMBgNVBAgMBVRva3lvMQ0wCwYDVQQHDARVZW5v
MRQwEgYDVQQKDAtUZXN0U2VydmljZTEYMBYGA1UEAwwPTXlUZXN0U2VydmljZUNB
MB4XDTI1MDkyNDA1NDkzNFoXDTM1MDkyMjA1NDkzNFowXDELMAkGA1UEBhMCSlAx
DjAMBgNVBAgMBVRva3lvMQ0wCwYDVQQHDARVZW5vMRQwEgYDVQQKDAtUZXN0U2Vy
dmljZTEYMBYGA1UEAwwPTXlUZXN0U2VydmljZUNBMIIBIjANBgkqhkiG9w0BAQEF
AAOCAQ8AMIIBCgKCAQEAvqP7GuNgv5umIIXK+QqT2auq56x1oSAA+oP4Fmp+sjcO
e08QES/LlbXesVRYHHV624qInpdEKTwuENxZi0+mkm5zO09GFlQiKvas3YvN5Ecq
aWVoPWHezcXt2K+ogNU1rjPqgENvpKnWtzZPGIHiKnN0/taUR7AotxPg4wV1QVuv
EgavXqIFGBhH+Os+6HnCls3XukmMBu3YTxgB6wwYUHbXqzBQNivugXU1AFx1a7tI
GRCGEAJiJ//g1mTu8Ji2/XvN/QpkSv5GgjwDToB07ZvRNSOarODOwT37Fr87Nfy1
IpIGcmbNcpJik+LNqpyfdUALuLyXAbjjYw7OML9HqwIDAQABo1MwUTAdBgNVHQ4E
FgQUKazWF2yeXfg6aTVhYFwTyT3idPQwHwYDVR0jBBgwFoAUKazWF2yeXfg6aTVh
YFwTyT3idPQwDwYDVR0TAQH/BAUwAwEB/zANBgkqhkiG9w0BAQsFAAOCAQEAa9Qc
TEzjkbIWczZ1h9xDriG8RcST0pmdTR2QMo/UeaJayNjtxp9rEuNVW8Xs7COENtPv
wfq/X3RLXh+YBYT2qC0qO1HuAoLOHchxKMEJBJo1QjVkNmJHtIq6sddzm16EmW0o
jz2dN3fp1S9PvLwKha8YOMX1R9eiwT7UHAXJjOud6xdF6T8veDvhrFKTsKxgKFkG
NoUiNw+rzOVCOuwgt1I7VcfIgA0dmjDz3RlK1nPXroZaeMKgLEAyLPh704h8vvDT
ZHgx+Nrf1b+TNgNLxpnmqFH4NWdezFSfvHHRBOY3sFvMJFU81fnk3QTZYnRwrsSD
M7eKGakIiHjPZDWOEg==
-----END CERTIFICATE-----"

VERIFY_KEY="-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA96yON3DXHHS7DMfe4h1W
VSyRJI/mkrKNgDmIboIGYYNcBALMblC3IRgE9gEpaGJ1Ll+At5yl4UF90iru+4Kr
/bPRRjnCUBIyBztX7TcRPlbH4zfvYHx9vJtUuckVELwzmVz5T1K6UgbXQOZv96Y6
PNcBnrc/M0zW7GWIqcktEMuSdcMlYdJ8VJYNA4sfpSQLJfZI+j964tjoVhr2JNsi
kInG6TZmNBjjPSqFtDzTLxC12ngxJQDUuN1IRqAcSchjWMFwY67voVpS39u2nFVj
rbnHFMU8bkmTF3WizL+6ZPKPU28+WrYxH4EIEhToyIhktCb9DSKUSiIGa6+KvpfN
ywIDAQAB
-----END PUBLIC KEY-----"

SIGN_KEY="-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQD3rI43cNccdLsM
x97iHVZVLJEkj+aSso2AOYhuggZhg1wEAsxuULchGAT2ASloYnUuX4C3nKXhQX3S
Ku77gqv9s9FGOcJQEjIHO1ftNxE+VsfjN+9gfH28m1S5yRUQvDOZXPlPUrpSBtdA
5m/3pjo81wGetz8zTNbsZYipyS0Qy5J1wyVh0nxUlg0Dix+lJAsl9kj6P3ri2OhW
GvYk2yKQicbpNmY0GOM9KoW0PNMvELXaeDElANS43UhGoBxJyGNYwXBjru+hWlLf
27acVWOtuccUxTxuSZMXdaLMv7pk8o9Tbz5atjEfgQgSFOjIiGS0Jv0NIpRKIgZr
r4q+l83LAgMBAAECggEALuPI0v82gokpBozqigWC3EJBQlpKDVjniDCcP0u3mIuN
hqbe/D2kxgutmMN0ivIk/EARdvGdyA0lnH4LW6uME06RXsm9m3ouZYcbKOplhddZ
JY/n7mzzQxtnSXsj1VTEMhNTkex4IOJxqzRVW13ppa4Q/PL1cKlqATxhyL8xHH4G
pmq8Q899T7OW7vLdysede68sjbA04fL/gaPNxPj5TpsPKvreIQRpziXDoJCalMp9
EUi0CbzpoVheahJlSi6In9byRxGauVIao+BgNh/NNYqVnj/Tp6X2YGnhN5UXYA+j
V4xMjmKFgHIFaptpUTudpyAZZnG/WQVKJDeixhscaQKBgQD9MpJw0cgLhRenwL0V
zJeMlt1OwnA4sbbmxUS67eAy31cZzUS6N2cF+2RaP0WjGSnZyxcXPA72HXnM8/dP
B5tX6ce9PJ0px7YtOnwwcjGMKqQsPALF9Uvm5FfuWlCdHaHzfUv2wUGS8ON6cJDW
qgufBrMynmtw8ZG1Wr+5MIiRgwKBgQD6alUzZrAJOwM/IYdIte3YISx4a69j0epc
Vh7Bzm3tQYF02nSFpMSKX8sQeQ5wFx4gjhGWJp3tn0xrWrsN44b0oG4zKE9QaRVJ
hCzBa/Ka+p/EsXc/kc9CSMuylJ20LtA0B5TEgYJ8QzzCA8BsRUc47+JkR3gO1N9w
jS5bPfyIGQKBgQC6f9Kv+UXBfoJDFUvxz6Zdbw6KIdxpVjWj2/BZRDgdILdWkQUr
qP1gwaBUfUB8918FRnu2qI1YqbN6zMUAWFkLM27lq80T5kABJpAtWx+13/7XekiM
qbcD1nQSZEH2yMnuwP8APa9gXcEhAeMdy1kOBPBfu6LmKXmrPLH15ZLiowKBgDgO
u7oA/+FhG43zZISLbY4XhwwCF0ZCRLOc98+s9YDKTD+rc7BDPVg4r42le+ztz+m7
xAYX6Py7z3Cs4/js+VYj3+eF25OFoqVNeHNoRewZtNBkZeyOKJaPE0KL8G3YmPU8
yTngQCSvLJfGHTpfm90MHmMSeLbhQo/AmyMD0ldpAoGBAJ9tQ3R33AYkkjCJSc/u
8X121N2+URZxuA23bMJH6OoJddtz8AFyKV36ihbVKrJ1/mcXkdZ9+WEszQaVsGsm
CvsxaZWlMj4yZoVCx7ZqrFx17AThlxCpi7rFoFZbkk+M9+RX6U8d8r39qyfqjJFp
0kaUPHgv1Qgvn5SYcebU+AQ4
-----END PRIVATE KEY-----"

# 1. Realm
echo "Step 1: Creating Realm..."
jq -n --arg name "$REALM_NAME" --arg ca "$CACERT" --arg vk "$VERIFY_KEY" --arg sk "$SIGN_KEY" \
  '{name: $name, title: "Quench Shop System", description: "Realm for quench environment (mirrored from forge)", cacert: $ca, deviceIdVerificationKey: $vk, deviceIdSigningKey: $sk, sessionTimeout: 2592000, disabled: true}' \
  | curl -s -X POST "$API_BASE_URL/realms" -H "Content-Type: application/json" -d @-

# 2. Zone
echo ""
echo "Step 2: Creating Zone..."
jq -n --arg name "$ZONE_NAME" \
  '{name: $name, title: ("Shop zone " + $name), description: "DNS zone configuration for quench."}' \
  | curl -s -X POST "$API_BASE_URL/realms/$REALM_NAME/zones" -H "Content-Type: application/json" -d @-

# 3. Subdomains and VirtualHosts
SUBS="www auth api check sirius betelgeuse procyon"
for sub in $SUBS; do
    echo ""
    echo "Step 3: Creating Subdomain & VirtualHost for '$sub'..."
    # Subdomain
    jq -n --arg name "$sub" \
      '{name: $name, title: $name, description: ("Subdomain configuration for " + $name)}' \
      | curl -s -X POST "$API_BASE_URL/realms/$REALM_NAME/zones/$ZONE_NAME/subdomains" -H "Content-Type: application/json" -d @-
    
    # VirtualHost
    jq -n --arg name "$sub" --arg r "$REALM_NAME" --arg z "$ZONE_NAME" \
      '{name: $name, title: $name, subdomain: ("urn:chip-in:subdomain:" + $r + ":" + $z + ":" + $name)}' \
      | curl -s -X POST "$API_BASE_URL/realms/$REALM_NAME/virtual-hosts" -H "Content-Type: application/json" -d @-
done

# 4. RoutingChain
echo ""
echo "Step 4: Creating RoutingChain..."
jq -n --arg dom "$ZONE_NAME" \
  '{name: "fruit-shop-route", title: "Quench Routing Chain", rules: [
    {match: ("hostname.equals(\"www." + $dom + "\")"), action: {type: "proxy", upstream: "http://127.0.0.1:9000"}},
    {match: ("hostname.equals(\"auth." + $dom + "\")"), action: {type: "proxy", upstream: "http://127.0.0.1:9001"}},
    {match: "request.path.starts_with(\"/api/\")", action: {type: "proxy", upstream: "http://127.0.0.1:9002", authScopeName: "shop_scope"}}
  ]}' | curl -s -X POST "$API_BASE_URL/realms/$REALM_NAME/routing-chains" -H "Content-Type: application/json" -d @-

# 5. Hub
echo ""
echo "Step 5: Creating Hub..."
jq -n --arg n "hub1" --arg f "hub1.$ZONE_NAME" \
  '{name: $n, title: "Quench Hub", fqdn: $f, serverAddress: "0.0.0.0", serverPort: 4433, serverCert: "dummy-cert", serverCertKey: "dummy-key", attributes: {}}' \
  | curl -s -X POST "$API_BASE_URL/realms/$REALM_NAME/hubs" -H "Content-Type: application/json" -d @-

# 6. Services
SVCS="www api auth database"
for svc in $SVCS; do
    echo ""
    echo "Step 6: Creating Service '$svc'..."
    jq -n --arg n "$svc" --arg z "$ZONE_NAME" \
      '{name: $n, title: ($n + " service"), provider: ("urn:chip-in:end-point:" + $z + ":" + $n + "-server"), consumers: [("urn:chip-in:end-point:" + $z + ":" + $n + "-gateway")]}' \
      | curl -s -X POST "$API_BASE_URL/realms/$REALM_NAME/hubs/hub1/services" -H "Content-Type: application/json" -d @-
done

echo ""
echo "==> Quench realm build complete."
