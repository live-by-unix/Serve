**Security Policy for Serve**

**Overview

**Serve is a lightweight static file hoster. It exposes files over HTTP or HTTPS from a directory chosen by the user. Because Serve directly serves files from the filesystem, users must understand the security implications before deploying it in any environment.**

**Supported Versions**

**Serve is a simple tool and does not maintain long-term security branches. Only the latest release is supported. Users should always update to the newest version. Unless if a version is labled with a LTS tag in the release. **

**Security Considerations**

**1. Serve does not sandbox or isolate the directory it hosts. Any file inside the chosen directory may be served if requested.**
**2. Serve does not provide authentication or access control. Anyone who can reach the server can access the hosted files.**
**3. Serve should not be exposed directly to the public internet without a reverse proxy or firewall.**
**4. HTTPS mode uses self-signed certificates. These certificates provide encryption but do not establish trust. Browsers will show warnings.**
**5. Directory listings reveal filenames and structure. Use the -i flag carefully if index.html exists.**
**6. Running Serve with elevated privileges is not recommended. Use a normal user account.**
**7. Serve does not execute code, run scripts, or process dynamic content. It only serves static files.**

**Best Practices**

**1. Host only directories that are safe to expose.**
**2. Use a firewall to restrict access to trusted networks.**
**3. Place Serve behind a reverse proxy if deploying publicly.**
**4. Regenerate certificates if compromised.**
**5. Keep Serve updated to the latest version.**
**6. Avoid hosting sensitive or private data.**

**Reporting a Vulnerability**

**If you discover a security issue in Serve, please open a Github Issue on GitHub. Provide clear steps to reproduce the issue and any relevant details.**
