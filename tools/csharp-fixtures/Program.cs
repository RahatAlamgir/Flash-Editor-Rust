// Usage:
//   dotnet run --project tools/csharp-fixtures -- make <output folder>     writes tutorial.flash and features.flash
//   dotnet run --project tools/csharp-fixtures -- summary <file.flash>     prints what the C# reader sees

using System.IO;
using System.Reflection;
using System.Security.Cryptography;
using System.Text;
using FlashEditor.Core.Documents;
using FlashEditor.Core.Imaging;
using FlashEditor.Core.IO.Project;

internal static class Program
{
    [STAThread]
    private static int Main(string[] args)
    {
        if (args.Length < 2) { Console.Error.WriteLine("usage: make <folder> | summary <file.flash>"); return 2; }
        _ = new System.Windows.Application();          // lets the tutorial load its pictures from the C# resources
        FlashEditor.Core.Objects.ObjectRendering.Renderer = new FlashEditor.App.Rendering.ObjectRenderer();
        switch (args[0])
        {
            case "make": Make(args[1]); return 0;
            case "summary": Console.Write(Summary(args[1])); return 0;
            default: Console.Error.WriteLine("unknown command " + args[0]); return 2;
        }
    }

    private static void Make(string folder)
    {
        Directory.CreateDirectory(folder);
        var build = typeof(FlashEditor.App.Services.DemoProject).GetMethod("BuildPages", BindingFlags.NonPublic | BindingFlags.Static)
                    ?? throw new InvalidOperationException("DemoProject.BuildPages not found");
        var tutorial = (IReadOnlyList<ProjectPage>)build.Invoke(null, null)!;
        ProjectWriter.Save(tutorial, 0, Path.Combine(folder, "tutorial.flash"));
        ProjectWriter.Save(FeaturePages(), 1, Path.Combine(folder, "features.flash"));
        Console.WriteLine("written to " + Path.GetFullPath(folder));
    }

    /// <summary>Two pages with the plain layer features: off-canvas pixels, masks, adjustment layers, blend modes...</summary>
    private static List<ProjectPage> FeaturePages()
    {
        // Page 1: 640 x 400
        var doc = Document.CreateBlank(640, 400, new ColorRgba(250, 245, 230));
        var layers = doc.Layers.ToList();

        var moved = new Layer("Off canvas", 640, 400);                      // a red block half outside on the right
        var block = new PixelBuffer(200, 120); block.Fill(new ColorRgba(220, 40, 40));
        moved.RestoreState(LayerState.Place(block, 540, 40, 640, 400));
        layers.Add(moved);

        var masked = new Layer("Masked", 640, 400);                          // blue, left half hidden by a mask
        masked.Pixels.Fill(new ColorRgba(30, 90, 220, 200));
        var mask = new LayerMask(640, 400);
        for (int y = 0; y < 400; y++) for (int x = 0; x < 320; x++) mask.Alpha[y * 640 + x] = (byte)(x * 255 / 320);
        masked.SetMask(mask);
        masked.Opacity = 0.75;
        masked.BlendMode = BlendMode.Multiply;
        layers.Add(masked);

        var maskOff = new Layer("Mask switched off", 640, 400);
        for (int y = 300; y < 380; y++) for (int x = 20; x < 220; x++) maskOff.Pixels.SetPixel(x, y, new ColorRgba(20, 160, 90, 128));
        maskOff.SetMask(new LayerMask(640, 400, 0));
        maskOff.MaskEnabled = false;
        maskOff.IsLocked = true;
        layers.Add(maskOff);

        var hidden = new Layer("Hidden", 640, 400) { IsVisible = false, BlendMode = BlendMode.SoftLight };
        hidden.Pixels.Fill(new ColorRgba(255, 255, 0));
        layers.Add(hidden);

        layers.Add(new Layer("Levels", 640, 400) { Adjustment = new AdjustmentStyle(Kind: AdjustmentKind.Levels, LevelsBlack: 12, LevelsGamma: 1.4, OutputWhite: 240) });
        layers.Add(new Layer("Curves", 640, 400)
        {
            Adjustment = new AdjustmentStyle(Kind: AdjustmentKind.Curves, Curve: new[] { new CurvePoint(0, 0), new CurvePoint(90, 130), new CurvePoint(255, 255) },
                CurveRed: new[] { new CurvePoint(0, 20), new CurvePoint(255, 255) }),
        });
        layers.Add(new Layer("Warm", 640, 400) { Adjustment = new AdjustmentStyle(Brightness: 5, Contrast: 10, Saturation: -20, Hue: 15, Temperature: 30, Tint: -10), Opacity = 0.5 });
        doc.SetLayers(layers, masked);
        var page1 = new ProjectPage("Features", doc, new double[] { 100, 320.5 }, new double[] { 200 });

        // Page 2: a different size, one picture layer
        var doc2 = Document.CreateBlank(300, 500, ColorRgba.Transparent);
        for (int y = 0; y < 500; y++) for (int x = 0; x < 300; x++) doc2.Layers[0].Pixels.SetPixel(x, y, new ColorRgba((byte)x, (byte)(y / 2), (byte)(x ^ y), (byte)(255 - y / 4)));
        var page2 = new ProjectPage("Tall page", doc2);
        return new List<ProjectPage> { page1, page2 };
    }

    /// <summary>What the C# reader sees, as text (two files with the same summary hold the same project).</summary>
    private static string Summary(string path)
    {
        var content = ProjectReader.OpenProject(path);
        var sb = new StringBuilder();
        sb.AppendLine($"pages {content.Pages.Count}, active {content.ActivePage}");
        foreach (var page in content.Pages)
        {
            var doc = page.Document;
            sb.AppendLine($"page '{page.Name}' {doc.Width}x{doc.Height} active={doc.ActiveLayerIndex} vguides=[{string.Join(",", page.VerticalGuides)}] hguides=[{string.Join(",", page.HorizontalGuides)}]");
            foreach (var layer in doc.Layers)
            {
                var (full, fx, fy) = layer.GetFullContent();
                string adjustment = layer.Adjustment is { } a ? System.Text.Json.JsonSerializer.Serialize(a) : "-";
                string obj = layer.Object is { } o ? $"{o.GetType().Name} box={o.Box}" : "-";
                string maskHash = layer.Mask is { } m ? Hash(m.Alpha) : "-";
                sb.AppendLine($"  layer '{layer.Name}' opacity={layer.Opacity:0.###} visible={layer.IsVisible} locked={layer.IsLocked} blend={layer.BlendMode}");
                sb.AppendLine($"    pixels={Hash(layer.Pixels.Data)} full={full.Width}x{full.Height}@{fx},{fy}:{Hash(full.Data)} restore={(layer.RestoreSource is { } r ? Hash(r.Data) : "-")}");
                sb.AppendLine($"    mask={maskHash} maskEnabled={layer.MaskEnabled} object={obj}");
                sb.AppendLine($"    adjustment={adjustment}");
            }
        }
        return sb.ToString();
    }

    private static string Hash(byte[] data) => Convert.ToHexString(SHA256.HashData(data))[..12];
}
