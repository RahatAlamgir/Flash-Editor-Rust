; =============================================================================
; Flash Editor installer (Inno Setup 6) - the only edition: Standard.
;
; The program (flash-editor.exe, installed as "Flash Editor.exe") plus the three AI runtime DLLs. The AI models are NOT
; included: during setup you can tick the ones to download (checked with SHA-256), anything skipped is offered by the AI
; tools on first use, or in Settings > AI Models.
; Build with build-installer.ps1 (it builds the release exe and passes /DAppVersion).
;
; EVERYTHING lives in the install folder: the program, the AI runtime and the models (in "models\"), the settings (in
; "settings\"). Nothing is written to AppData. Uninstalling removes the whole folder, including downloaded models.
; =============================================================================

#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif

#define AppName      "Flash Editor"
#define AppExe       "Flash Editor.exe"
#define BuildDir     "build\app"

[Setup]
AppId={{D3A0C7F1-5B8E-4F6A-8C2D-71E94A0B6F35}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=Flash Editor
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\{#AppExe}
SetupIconFile=..\assets\icons\app\FlashEditor.ico
UninstallDisplayName={#AppName}
OutputDir=output
OutputBaseFilename=FlashEditor-Setup-{#AppVersion}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
#ifdef TestInstall
; Test builds only (no admin prompt): install into a folder of your choice with /DIR=...
PrivilegesRequired=lowest
#else
PrivilegesRequired=admin
#endif
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Dirs]
; The program downloads AI models into this folder, so normal users may write here.
Name: "{app}\models"; Permissions: users-modify
; The program keeps its settings, log and autosave here (also written by normal users).
Name: "{app}\settings"; Permissions: users-modify

[Files]
Source: "{#BuildDir}\flash-editor.exe"; DestDir: "{app}"; DestName: "{#AppExe}"; Flags: ignoreversion
; The AI runtime (ONNX Runtime + DirectML). Loaded only when an AI tool is first used.
Source: "{#BuildDir}\onnxruntime.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#BuildDir}\onnxruntime_providers_shared.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#BuildDir}\DirectML.dll"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Models downloaded later by the program are not in the installer's file list: remove them too,
; so uninstalling leaves nothing behind.
Type: filesandordirs; Name: "{app}\models"
Type: filesandordirs; Name: "{app}\settings"
Type: dirifempty; Name: "{app}"

[Code]
// ---- Optional AI model download (Standard edition) -------------------------------------------
// A page after the task list lets the user tick the AI features to download. The downloads run at the end of the
// installation (after the files are copied), are verified with SHA-256 and saved in the "models" folder of the
// install folder, where the program finds them. A failed or skipped download never fails the installation: the
// program offers it again when the feature is first used (or in Settings > AI Models). Silent installs download nothing.

const
  UrlIsNet   = 'https://github.com/danielgatis/rembg/releases/download/v0.0.0/isnet-general-use.onnx';
  UrlLama    = 'https://huggingface.co/Carve/LaMa-ONNX/resolve/main/lama_fp32.onnx';
  UrlEsrgan  = 'https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-models/models/real_esrgan_x4plus/releases/v0.66.0/real_esrgan_x4plus-onnx-float.zip';
  UrlSamEnc  = 'https://huggingface.co/Acly/MobileSAM/resolve/main/mobile_sam_image_encoder.onnx';
  UrlSamDec  = 'https://huggingface.co/Acly/MobileSAM/resolve/main/sam_mask_decoder_single.onnx';

var
  ModelsPage: TInputOptionWizardPage;
  DownloadPage: TDownloadWizardPage;

procedure InitializeWizard;
begin
  ModelsPage := CreateInputOptionPage(wpSelectTasks,
    'AI features (optional)',
    'Download the AI models you want to use now',
    'The AI tools need a model file each. Tick the ones to download now (about 470 MB in total, needs internet). ' +
    'Anything you skip is offered later, the first time you use that tool, or in Settings > AI Models.',
    False, False);
  ModelsPage.Add('Remove Background and Select Subject (170 MB)');
  ModelsPage.Add('Remove / Fill and Remove Brush (198 MB)');
  ModelsPage.Add('AI Upscale / Clean Up (62 MB)');
  ModelsPage.Add('Click Select (44 MB)');
  ModelsPage.Values[0] := True;
  ModelsPage.Values[1] := True;
  ModelsPage.Values[2] := True;
  ModelsPage.Values[3] := True;
  DownloadPage := CreateDownloadPage('Downloading AI models', 'Please wait while the AI models are downloaded.', nil);
end;

function ShouldSkipPage(PageID: Integer): Boolean;
begin
  Result := (PageID = ModelsPage.ID) and WizardSilent;
end;

procedure DownloadModels;
var
  Models, Tmp: String;
  Failed: Boolean;
begin
  Models := ExpandConstant('{app}\models');
  Tmp := ExpandConstant('{tmp}');
  ForceDirectories(Models);
  DownloadPage.Clear;
  if ModelsPage.Values[0] then
    DownloadPage.Add(UrlIsNet, 'isnet-general-use.onnx', '60920e99c45464f2ba57bee2ad08c919a52bbf852739e96947fbb4358c0d964a');
  if ModelsPage.Values[1] then
    DownloadPage.Add(UrlLama, 'lama_fp32.onnx', '1faef5301d78db7dda502fe59966957ec4b79dd64e16f03ed96913c7a4eb68d6');
  if ModelsPage.Values[2] then
    DownloadPage.Add(UrlEsrgan, 'real_esrgan_x4plus.zip', '05aa31323c6a7e831493171cac970e398697709cae7e7ae8e0760e6da4d64ca4');
  if ModelsPage.Values[3] then
  begin
    DownloadPage.Add(UrlSamEnc, 'mobile_sam_image_encoder.onnx', '580f5fb648ea1062c0aabc26217aed56921985f03f0cbbd852bba81d760cc749');
    DownloadPage.Add(UrlSamDec, 'sam_mask_decoder_single.onnx', '93915fc7c993ab9d59ab8c9ccd3bce37f7509c81ab4150a74abd4d2abbd8570d');
  end;
  if not (ModelsPage.Values[0] or ModelsPage.Values[1] or ModelsPage.Values[2] or ModelsPage.Values[3]) then Exit;     { nothing ticked }

  Failed := False;
  DownloadPage.Show;
  try
    try
      DownloadPage.Download;
    except
      Failed := True;
      if DownloadPage.AbortedByUser then
        Log('AI model download cancelled by the user.')
      else
        SuppressibleMsgBox('The AI models could not be downloaded:' + #13#10 + GetExceptionMessage + #13#10#13#10 +
          'Flash Editor is installed. Each AI tool offers to download its model the first time you use it ' +
          '(or use Settings > AI Models).', mbInformation, MB_OK, IDOK);
    end;
  finally
    DownloadPage.Hide;
  end;
  if Failed then Exit;

  if ModelsPage.Values[0] then FileCopy(Tmp + '\isnet-general-use.onnx', Models + '\isnet-general-use.onnx', False);
  if ModelsPage.Values[1] then FileCopy(Tmp + '\lama_fp32.onnx', Models + '\lama_fp32.onnx', False);
  // The upscaling model is a zip: it is only copied here (already checked with SHA-256); the program unpacks it the first time it is used.
  if ModelsPage.Values[2] then FileCopy(Tmp + '\real_esrgan_x4plus.zip', Models + '\real_esrgan_x4plus.zip', False);
  if ModelsPage.Values[3] then
  begin
    FileCopy(Tmp + '\mobile_sam_image_encoder.onnx', Models + '\mobile_sam_image_encoder.onnx', False);
    FileCopy(Tmp + '\sam_mask_decoder_single.onnx', Models + '\sam_mask_decoder_single.onnx', False);
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and (not WizardSilent) then DownloadModels;
end;
