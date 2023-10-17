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
        File.WriteAllText(Path.Combine(Application.dataPath, "materials.json"), output);
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
        File.WriteAllText(Path.Combine(Application.dataPath, "textures.json"), output);
    }
}
