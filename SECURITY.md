# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of FROST Protocol seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### Reporting Process

1. **DO NOT** open a public issue on GitHub
2. Send an email to [security@yourdomain.com] with:
   - A detailed description of the vulnerability
   - Steps to reproduce the issue
   - Potential impact
   - Suggested fix (if any)

### What to Expect

1. **Initial Response**: We will acknowledge your report within 48 hours
2. **Updates**: We will keep you informed about the progress
3. **Resolution**: Once fixed, we will:
   - Notify you
   - Release a security advisory
   - Issue a patch release if necessary

## Security Considerations

### Protocol Security
- All network communications are encrypted using libp2p's noise protocol
- State proofs are cryptographically verified
- Circuit breakers protect against network attacks
- Resource limits prevent DoS attacks

### Best Practices
1. Keep your FROST Protocol implementation up to date
2. Use secure key management practices
3. Monitor system metrics and alerts
4. Follow security advisories

## Security Features

- Encrypted P2P communication
- Proof verification system
- Circuit breaker protection
- Resource limiting
- Error detection
- Audit logging

## Known Security Limitations

- Some advanced security features planned for future releases
- Chain-specific security measures must be implemented separately
- Performance vs. security tradeoffs may exist in some components

## Acknowledgments

We would like to thank all security researchers who have helped improve FROST Protocol's security. Contributors will be acknowledged (with permission) in our security advisories. 