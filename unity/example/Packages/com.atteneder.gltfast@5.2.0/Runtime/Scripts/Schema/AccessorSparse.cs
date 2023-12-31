// SPDX-FileCopyrightText: 2023 Unity Technologies and the glTFast authors
// SPDX-License-Identifier: Apache-2.0

namespace GLTFast.Schema
{

    /// <summary>
    /// Sparse property of a glTF <seealso cref="Accessor"/>
    /// </summary>
    [System.Serializable]
    public class AccessorSparse
    {
        /// <summary>
        /// Number of entries stored in the sparse array.
        /// <minimum>1</minimum>
        /// </summary>
        public int count;

        /// <summary>
        /// Index array of size `count` that points to those accessor attributes that
        /// deviate from their initialization value. Indices must strictly increase.
        /// </summary>
        public AccessorSparseIndices indices;

        /// <summary>
        /// "Array of size `count` times number of components, storing the displaced
        /// accessor attributes pointed by `indices`. Substituted values must have
        /// the same `componentType` and number of components as the base accessor.
        /// </summary>
        public AccessorSparseValues values;

        internal void GltfSerialize(JsonWriter writer)
        {
            writer.AddObject();
            writer.AddProperty("count", count);
            if (indices != null)
            {
                writer.AddProperty("indices");
                indices.GltfSerialize(writer);
            }
            if (values != null)
            {
                writer.AddProperty("values");
                values.GltfSerialize(writer);
            }
            writer.Close();
        }

    }
}
