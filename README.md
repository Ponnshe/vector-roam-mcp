# Multi-Language MCP Ecosystem: Semantic Knowledge Retrieval

This repository documents the development of a hybrid **Model Context Protocol (MCP)** architecture designed to bridge structured local notes (Org-roam/Anytype) with high-performance semantic retrieval using **Qdrant**.

## 🏗️ Project Structure

To maintain a clean separation of concerns, the project is organized as follows:

```text
.
├── docker-compose.yml         # Qdrant persistence layer
├── flake.nix                  # Unified NixOS development environment
├── flake.lock
├── python_notes_server/       # Python MCP Server (Notes management)
│   ├── main.py
│   ├── pyproject.toml
│   └── uv.lock
└── rust_vector_engine/        # Rust MCP Server (Embeddings & Qdrant)
    ├── Cargo.toml
    └── src/
        └── main.rs
```


# Core Concepts: Vector Search & Latent Representations
The following theoretical framework guides the engineering decisions of this project.

## 1. Embeddings and the Latent Space
AI models do not "use" Vector Search; they perform Feature Extraction to generate Embeddings. This process maps unstructured data (text, images, audio) into a High-Dimensional Latent Space (a manifold) through Inference.

The "Essence": Each vector is a fixed-length list of floating-point numbers. The "conceptual essence" is simply the coordinate of a data point within this space.

Non-Interpretability: Dimensions are latent, meaning they do not have human-readable names like "furry" or "domestic." The meaning is distributed across the entire vector.

## 2. Contextual Mapping (Distributional Semantics)
Models learn through the Distributional Hypothesis: words that appear in similar contexts share similar meanings.
- They don't "understand" a forest; they calculate that the tokens forest, trees, and green have high co-occurrence probabilities.
- The mapping process clusters these related tokens in the same neighborhood of the latent space.

## 3. Vector Arithmetic and Semantic Relationships
The spatial organization allows for Linear Relational Mapping. Because relationships are encoded as directions and distances (offsets), we can perform arithmetic on concepts:
- *Logic:*

$$\vec{v}_{\text{bark}} \approx \vec{v}_{\text{meow}} + (\vec{v}_{\text{dog}} - \vec{v}_{\text{cat}})$$

- The term ($`\vec{v}_{\text{meow}} \\- \vec{v}_{\text{cat}}`$) is the "sound-to-species" offset. Adding it to $\vec{v}_{\text{dog}}$ moves us to the corresponding sound for that species.

## 4. Perplexity and Statistical Anomalies
In the phrase "my cat barks," a model detects an anomaly not through logic, but through Statistical Distance.

- In the training corpus, the transition probability between the cat vector and the bark vector is near zero.

- The model perceives high Perplexity: the sequence is mathematically "unexpected" because the vectors are located in distant, non-correlated regions of the manifold.

## 5. Retrieval vs. Inference
It is critical to distinguish between the Encoder and the Vector Database:
- *The Encoder (LLM/Transformer):* Takes the input and generates the vector (Inference).
- *The Vector Database (Qdrant):* Stores millions of vectors and performs the search.
- *The Metric:* Instead of exact keyword matching, it uses Cosine Similarity to measure the angular proximity between the query vector and stored vectors:

  $$\text{cosine\\_sim}(\mathbf{A}, \mathbf{B}) = \frac{\mathbf{A} \cdot \mathbf{B}}{\|\mathbf{A}\| \|\mathbf{B}\|}$$

## 6. Search Optimization (ANN)
For massive datasets, calculating the distance against every single vector is computationally expensive ($O(n)$). Engineering solutions use Approximate Nearest Neighbors (ANN) algorithms, like HNSW (Hierarchical Navigable Small Worlds), to find the "closest" concepts in logarithmic time.

---

# Tech Stack
- Languages: Python 3.13 (Notes Logic), Rust (Vector Engine).

- Environment: NixOS with flake.nix for reproducible builds.

- Database: Qdrant (Vector Database).

- Package Management: uv (Python), cargo (Rust).

- Protocols: Model Context Protocol (MCP).

---

# Getting Started
## 1. Environment Setup
Ensure you are using Nix with flakes enabled.

```bash
nix develop
```

## 2. Infrastructure
Spin up the local Qdrant instance:
```bash
docker compose up -d
```

##  3. Running the MCP Servers
To debug and inspect the servers during development, use the MCP inspector.

**For the Python Server:**
```bash
cd python_notes_server
uv run mcp dev main.py
```
**For the rust server:**
```bash
cd rust_vector_engine
cargo build
mcp dev ./target/debug/rust_vector_engine
```
