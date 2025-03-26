; Inno docs: https://jrsoftware.org/ishelp/
; Preprocessor docs: https://jrsoftware.org/ispphelp/
#pragma verboselevel 9

; Extract version from one of the exe's
#define Exe "dist\wsl-tools\recycle.exe"
#ifnexist Exe
  #pragma error Exe + " does not exist"
#endif
#define FileVersion GetStringFileInfo(Exe, "FileVersion")
#define ProductVersion GetStringFileInfo(Exe, "ProductVersion")
#define Version Copy(ProductVersion, 1, Pos("+", ProductVersion) - 1)
#pragma message "Version is " + Version

[Setup]
AppCopyright=Copyright © Max Kagamine
AppId={{C5EB6945-50E0-46B8-A7C8-883105DCC39}
AppName=wsl-tools
AppPublisher=Max Kagamine
AppPublisherURL=https://github.com/maxkagamine/wsl-tools
AppSupportURL=https://github.com/maxkagamine/wsl-tools/issues
AppUpdatesURL=https://github.com/maxkagamine/wsl-tools/releases
AppVerName=wsl-tools {#Version}
AppVersion={#Version}
ArchitecturesAllowed=x64os
ArchitecturesInstallIn64BitMode=x64compatible
DefaultDirName={autopf}\wsl-tools
DisableProgramGroupPage=yes
LicenseFile=LICENSE.txt
OutputBaseFilename=wsl-tools-installer
OutputDir=dist
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
SolidCompression=yes
VersionInfoProductTextVersion={#ProductVersion}
VersionInfoVersion={#FileVersion}
WizardStyle=classic

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"

[Files]
Source: "dist\wsl-tools\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
