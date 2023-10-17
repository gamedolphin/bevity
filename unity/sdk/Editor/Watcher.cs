using UnityEditor;
using System.Collections.Generic;
using System.Collections.Concurrent;
using UnityEngine;
using System.Text;
using Newtonsoft.Json;
using System.IO;
using UnityEngine.SceneManagement;

[System.Serializable]
public struct ChangeAck
{
    public ulong object_id;
}

[System.Serializable]
public struct ChangeObject
{
    public ulong object_id;
    public string serialized;
}


public static class Watcher
{
    public const int UPDATE = 0;
    public const int ACK = 1;

    private static Dictionary<int, ChangeGameObjectOrComponentPropertiesEventArgs> changesToBeSent = new Dictionary<int, ChangeGameObjectOrComponentPropertiesEventArgs>();
    private static ConcurrentQueue<string> incomingMessages = new ConcurrentQueue<string>();
    private static Dictionary<ulong, string> incomingChanges = new Dictionary<ulong, string>();
    private static Dictionary<ulong, string> oldValues = new Dictionary<ulong, string>();
    private static Dictionary<ulong, int> pendingChanges = new Dictionary<ulong, int>();

    private static StreamWriter stdin = null;
    private static RuntimeWatcher runtimeWatcher;

    public static void Start(StreamWriter processStdin)
    {
        changesToBeSent.Clear();
        incomingMessages.Clear();
        pendingChanges.Clear();

        ObjectChangeEvents.changesPublished += ChangesPublished;
        var obj = new GameObject("RuntimeWatcher");
        GameObject.DontDestroyOnLoad(obj);
        runtimeWatcher = obj.AddComponent<RuntimeWatcher>();
        runtimeWatcher.onUpdate += OnUpdate;
        stdin = processStdin;
    }

    public static void IncomingChange(string serialized)
    {
        incomingMessages.Enqueue(serialized);
    }

    public static void Stop()
    {
        ObjectChangeEvents.changesPublished -= ChangesPublished;
        changesToBeSent.Clear();
        incomingMessages.Clear();
        pendingChanges.Clear();

        stdin = null;
    }

    private static void OnUpdate()
    {
        SendChanges();
        HandleChanges();
    }

    private static void HandleChanges()
    {
        while (incomingMessages.TryDequeue(out var message))
        {
            var split = message.Split("|");
            var action = int.Parse(split[1]);
            var data = split[2];

            switch (action)
            {
                case 0:
                    var changes = JsonConvert.DeserializeObject<ChangeObject[]>(data);
                    foreach (var change in changes)
                    {
                        incomingChanges[change.object_id] = change.serialized;
                    }
                    break;
                case 1:
                    var acks = JsonConvert.DeserializeObject<ChangeAck[]>(data);
                    foreach (var ack in acks)
                    {
                        if (pendingChanges.TryGetValue(ack.object_id, out var count))
                        {
                            var final = count - 1;
                            if (final == 0)
                            {
                                pendingChanges.Remove(ack.object_id);
                            }
                            else
                            {
                                pendingChanges[ack.object_id] = final;
                            }
                        }
                    }
                    break;
            }
        }

        var activeScene = SceneManager.GetActiveScene().path;
        var guid = AssetDatabase.AssetPathToGUID(activeScene);
        foreach (var (k, v) in incomingChanges)
        {
            if (pendingChanges.TryGetValue(k, out var count) && count > 0)
            {
                continue;
            }

            var globalObjectId = new GlobalObjectId();
            GlobalObjectId.TryParse($"GlobalObjectId_V1-2-{guid}-{k}-0", out globalObjectId);
            var instanceId = GlobalObjectId.GlobalObjectIdentifierToInstanceIDSlow(globalObjectId);
            var goOrComponent = EditorUtility.InstanceIDToObject(instanceId);
            if (goOrComponent is Component comp)
            {
                oldValues[k] = EditorJsonUtility.ToJson(comp);
            }
            object boxedStruct = goOrComponent;
            try
            {
                EditorJsonUtility.FromJsonOverwrite(v, boxedStruct);
                goOrComponent = boxedStruct as Object;
            }
            catch
            {
                Debug.Log($"Failed to deserialize : {v}");
            }
        }

        incomingChanges.Clear();
    }

    private static void SendChanges()
    {
        var output = new StringBuilder();
        var changeList = new List<ChangeObject>();
        foreach (var (instanceId, changeGameObjectOrComponent) in changesToBeSent)
        {
            var goOrComponent = EditorUtility.InstanceIDToObject(instanceId);
            if (goOrComponent is GameObject go)
            {
                continue;
            }
            else if (goOrComponent is Component component)
            {
                var componentId = GlobalObjectId.GetGlobalObjectIdSlow(component).targetObjectId;
                string serialized;
                if (oldValues.TryGetValue(componentId, out var comp))
                {
                    // actual editor value got overwritten by some bullshit maybe
                    serialized = comp;
                }
                else
                {
                    serialized = EditorJsonUtility.ToJson(component);
                }

                changeList.Add(new ChangeObject { object_id = GlobalObjectId.GetGlobalObjectIdSlow(component.gameObject).targetObjectId, serialized = serialized });
                if (pendingChanges.TryGetValue(componentId, out var count))
                {
                    pendingChanges[componentId] = count + 1;
                }
                else
                {
                    pendingChanges[componentId] = 1;
                }
            }
        }

        if (changeList.Count > 0)
        {
            output.Append($"EDITOR_CHANGE|{UPDATE}|");
            output.Append(JsonConvert.SerializeObject(changeList));
            if (stdin != null)
            {
                stdin.WriteLine(output);
            }
        }

        changesToBeSent.Clear();
        oldValues.Clear();
    }

    private static void ChangesPublished(ref ObjectChangeEventStream stream)
    {
        for (int i = 0; i < stream.length; ++i)
        {
            var type = stream.GetEventType(i);
            switch (type)
            {
                case ObjectChangeKind.ChangeGameObjectOrComponentProperties:
                    stream.GetChangeGameObjectOrComponentPropertiesEvent(i, out var changeGameObjectOrComponent);
                    changesToBeSent.Add(changeGameObjectOrComponent.instanceId, changeGameObjectOrComponent);
                    break;
            }
        }
    }
}
