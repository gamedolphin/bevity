using UnityEditor;
using System.Collections.Generic;
using System.IO;
using UnityEngine;
using Newtonsoft.Json;

public class UnityAssetLister : AssetPostprocessor
{
    static void OnPostprocessAllAssets(string[] importedAssets, string[] deletedAssets, string[] movedAssets, string[] movedFromAssetPaths, bool didDomainReload)
    {
        ProcessMaterials();
        ProcessTextures();
        ProcessAllAssets();
    }

    private static void ProcessMaterials()
    {
        var guids = AssetDatabase.FindAssets("t:material", new[] { "Assets/Materials" });
        var json = new Dictionary<string, string>();
        foreach (string guid in guids)
        {
            json.Add(guid, AssetDatabase.GUIDToAssetPath(guid));
        }

        var output = JsonConvert.SerializeObject(json);
        var cargoPath = Path.Combine(Application.dataPath, BevitySettings.Instance.CargoToml);
        var workingDirectory = Path.GetDirectoryName(cargoPath);
        File.WriteAllText(Path.Combine(workingDirectory, "materials.json"), output);
    }

    private static void ProcessTextures()
    {
        var guids = AssetDatabase.FindAssets("t:texture2D", new[] { "Assets/Textures" });
        var json = new Dictionary<string, string>();
        foreach (string guid in guids)
        {
            json.Add(guid, AssetDatabase.GUIDToAssetPath(guid));
        }

        var output = JsonConvert.SerializeObject(json);
        var cargoPath = Path.Combine(Application.dataPath, BevitySettings.Instance.CargoToml);
        var workingDirectory = Path.GetDirectoryName(cargoPath);
        File.WriteAllText(Path.Combine(workingDirectory, "textures.json"), output);
    }

    private static void ProcessAllAssets()
    {
        var guids = AssetDatabase.FindAssets("", new[] { "Assets/Models", "Assets/Textures", "Assets/Materials", "Assets/Prefabs" });
        var json = new Dictionary<string, string>();
        foreach (string guid in guids)
        {
            json.Add(guid, AssetDatabase.GUIDToAssetPath(guid));
        }

        var output = JsonConvert.SerializeObject(json);
        var cargoPath = Path.Combine(Application.dataPath, BevitySettings.Instance.CargoToml);
        var workingDirectory = Path.GetDirectoryName(cargoPath);
        File.WriteAllText(Path.Combine(workingDirectory, "all.json"), output);
    }

    // private static void ProcessModels()
    // {
    //     var guids = AssetDatabase.FindAssets("t:", new[] { "Assets/Textures" });
    //     var json = new Dictionary<string, string>();
    //     foreach (string guid in guids)
    //     {
    //         json.Add(guid, AssetDatabase.GUIDToAssetPath(guid));
    //     }

    //     var output = JsonConvert.SerializeObject(json);
    //     File.WriteAllText(Path.Combine(Application.dataPath, "textures.json"), output);
    // }
}
