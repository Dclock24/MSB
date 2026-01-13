# NETWORK SECURITY TOPOLOGY - DETAILED DIAGRAM

## HOME NETWORK SECURITY ARCHITECTURE
```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                    EXTERNAL INTERNET                                    │
│                                                                                        │
│  THREAT ACTORS           LEGITIMATE SERVICES              MONITORING ATTEMPTS          │
│  ❌ Blocked              ✅ Allowed                       ⚠️  Detected                 │
└────────────────────────────────────────┬───────────────────────────────────────────────┘
                                         │
                                         │ WAN CONNECTION
                                         │ ISP: Comcast/Xfinity
                                         │
                              ╔══════════▼═══════════╗
                              ║   ROUTER FIREWALL    ║
                              ║     10.0.0.1         ║
                              ║  MAC: 10:36:aa:      ║
                              ║      4:12:b3         ║
                              ╠══════════════════════╣
                              ║ • NAT Protection     ║
                              ║ • DHCP Server        ║
                              ║ • DNS Forwarding     ║
                              ║ • Port Forwarding: 0 ║
                              ╚══════════╤═══════════╝
                                         │
                         ┌───────────────┴───────────────┐
                         │   LOCAL AREA NETWORK (LAN)    │
                         │      10.0.0.0/24 Subnet       │
                         │    255.255.255.0 Netmask      │
                         └───────────────┬───────────────┘
                                         │
    ┌────────────────┬───────────────────┼───────────────────┬─────────────────┐
    │                │                   │                   │                 │
╔═══▼═══╗       ╔════▼════╗         ╔═══▼════╗        ╔════▼════╗      ╔══════▼══════╗
║ YOUR  ║       ║ DEVICE  ║         ║ DEVICE ║        ║ DEVICE  ║      ║   OTHER     ║
║ MACBOOK║       ║ .0.9    ║         ║ .0.13  ║        ║ .0.20   ║      ║   DEVICES   ║
╠════════╣       ╠═════════╣         ╠════════╣        ╠═════════╣      ╠═════════════╣
║ .0.39  ║       ║94:e6:ba:║         ║44:6f:f8║        ║6e:de:83:║      ║ Inactive    ║
║1e:38:9b║       ║f7:90:d8 ║         ║d1:56:2d║        ║f3:d7:b5 ║      ║ .0.2-.0.255 ║
╚════════╝       ╚═════════╝         ╚════════╝        ╚═════════╝      ╚═════════════╝
    │
    │
╔═══▼════════════════════════════════════════════════════════════════════════╗
║                        YOUR DEVICE - DETAILED VIEW                         ║
║                           IP: 10.0.0.39                                   ║
╠════════════════════════════════════════════════════════════════════════════╣
║                                                                            ║
║  LISTENING SERVICES                    ACTIVE CONNECTIONS                 ║
║  ┌──────────────────┐                 ┌──────────────────────────┐       ║
║  │ TCP PORTS:       │                 │ OUTBOUND:                │       ║
║  │ • 5000 - AirPlay │                 │ • 18 HTTPS (443)        │       ║
║  │ • 7000 - Control │                 │ • 1 Local (7100)        │       ║
║  │ • 63709 - Handoff│                 │ • 0 Suspicious          │       ║
║  └──────────────────┘                 └──────────────────────────┘       ║
║                                                                            ║
║  ┌──────────────────┐                 ┌──────────────────────────┐       ║
║  │ UDP PORTS:       │                 │ SECURITY STATUS:         │       ║
║  │ • 5353 - mDNS    │                 │ ✅ No Malware           │       ║
║  │ • 49616 - Share  │                 │ ✅ No Backdoors         │       ║
║  │ • 52524 - Sync   │                 │ ✅ No APTs              │       ║
║  └──────────────────┘                 │ ⚠️  Firewall OFF         │       ║
║                                        └──────────────────────────┘       ║
╚════════════════════════════════════════════════════════════════════════════╝
```

## PORT SECURITY MATRIX

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         PORT STATUS OVERVIEW                            │
├──────────┬────────────┬──────────┬─────────────┬──────────────────────┤
│   PORT   │  PROTOCOL  │  STATUS  │   SERVICE   │   SECURITY LEVEL     │
├──────────┼────────────┼──────────┼─────────────┼──────────────────────┤
│    22    │    TCP     │  CLOSED  │     SSH     │  ✅ SECURE           │
│    23    │    TCP     │  CLOSED  │   Telnet    │  ✅ SECURE           │
│    80    │    TCP     │  CLOSED  │    HTTP     │  ✅ SECURE           │
│   443    │    TCP     │  CLIENT  │    HTTPS    │  ✅ ENCRYPTED        │
│   445    │    TCP     │  CLOSED  │     SMB     │  ✅ SECURE           │
│  3389    │    TCP     │  CLOSED  │     RDP     │  ✅ SECURE           │
│  5000    │    TCP     │   OPEN   │   AirPlay   │  ✅ Apple Service    │
│  5353    │    UDP     │   OPEN   │    mDNS     │  ✅ Local Only       │
│  7000    │    TCP     │   OPEN   │   Control   │  ✅ Apple Service    │
│ 63709    │    TCP     │   OPEN   │   Handoff   │  ✅ Apple Service    │
└──────────┴────────────┴──────────┴─────────────┴──────────────────────┘
```

## TRAFFIC FLOW ANALYSIS

```
                    INBOUND TRAFFIC                  OUTBOUND TRAFFIC
                         ▼                                  ▲
┌──────────────────────────────────────────────────────────────────────┐
│                         PACKET INSPECTION                            │
├───────────────────────────────────────────────────────────────────────┤
│  BLOCKED (Firewall OFF - Router NAT Only)    ALLOWED                │
│  • Port Scans: N/A                           • HTTPS: 95%           │
│  • Malformed Packets: N/A                    • DNS: 3%              │
│  • Known Attack Signatures: N/A              • NTP: 1%              │
│  • Brute Force Attempts: N/A                 • Other: 1%            │
└───────────────────────────────────────────────────────────────────────┘
```

## CONNECTED ENDPOINTS ANALYSIS

```
┌────────────────────────────────────────────────────────────────────────┐
│                     ACTIVE CONNECTION ENDPOINTS                        │
├──────────────────────┬──────────────────┬──────────────────────────────┤
│     ENDPOINT         │       TYPE       │         ASSESSMENT           │
├──────────────────────┼──────────────────┼──────────────────────────────┤
│ 34.117.41.85         │  Google Cloud    │  ✅ SAFE - Known Service    │
│ 44.215.206.224       │  Amazon AWS      │  ✅ SAFE - Cloud Provider   │
│ 104.18.18.125        │  Cloudflare      │  ✅ SAFE - CDN             │
│ 17.248.207.23        │  Apple CDN       │  ✅ SAFE - Apple Services  │
│ 157.240.214.61       │  Meta/Facebook   │  ✅ SAFE - Social Media    │
│ 10.0.0.102           │  Local Device    │  ⚠️  MONITOR - Unknown     │
└──────────────────────┴──────────────────┴──────────────────────────────┘
```

## DNS RESOLUTION CHAIN

```
Your Device (10.0.0.39)
    │
    ├──► Router DNS Cache (10.0.0.1)
    │         │
    │         ├──► ISP DNS (75.75.75.75)
    │         │         │
    │         │         └──► Root DNS Servers
    │         │
    │         └──► ISP DNS (75.75.76.76)
    │                   │
    │                   └──► Root DNS Servers
    │
    └──► IPv6 DNS (2001:558:feed::1)
              │
              └──► IPv6 Root Servers
```

## SECURITY ZONES

```
┌─────────────────────────────────────────────────────────────────────┐
│                        SECURITY ZONE MAP                            │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  🔴 UNTRUSTED ZONE          │  🟡 DMZ              │  🟢 TRUSTED    │
│  ─────────────────          │  ───────            │  ────────────   │
│  • Internet                 │  • Router           │  • Your Mac     │
│  • Unknown IPs              │  • Gateway          │  • Local LAN    │
│  • Public Services          │                     │  • 10.0.0.0/24  │
│                             │                     │                 │
│  THREATS:                   │  MONITORING:        │  PROTECTED:     │
│  • Port Scans               │  • NAT Translation  │  • Full Access  │
│  • DDoS Attempts            │  • Connection Track │  • Trusted Apps │
│  • Malware C2               │  • Rate Limiting    │  • Local Share  │
│                             │                     │                 │
└──────────────────────────────────────────────────────────────────────┘
```

## RECOMMENDATIONS PRIORITY MATRIX

```
┌──────────────────────────────────────────────────────────────────────┐
│                    SECURITY IMPROVEMENTS ROADMAP                     │
├────────────┬──────────────────────────────┬────────────────────────┤
│  PRIORITY  │         ACTION ITEM          │     IMPACT LEVEL       │
├────────────┼──────────────────────────────┼────────────────────────┤
│    HIGH    │  Enable macOS Firewall       │  🔴🔴🔴 CRITICAL      │
│   MEDIUM   │  Implement DNS over HTTPS    │  🟡🟡 MODERATE         │
│   MEDIUM   │  Review Launch Agents        │  🟡🟡 MODERATE         │
│    LOW     │  Enable FileVault            │  🟢 MINOR              │
│    LOW     │  Setup Network Monitor       │  🟢 MINOR              │
└────────────┴──────────────────────────────┴────────────────────────┘
```

---
**Network Diagram Generated: January 3, 2026**
**Security Classification: UNCLASSIFIED**
