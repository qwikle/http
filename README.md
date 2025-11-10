# HTTP Server en Rust - Projet Éducatif

Un serveur HTTP 1.1 implémenté en Rust dans un but purement éducatif, permettant d'explorer les complexités de la mise en place d'un protocole réseau.

## 🎯 Objectif du Projet

Ce projet a été développé pour :
- **Comprendre en profondeur** le fonctionnement du protocole HTTP 1.1
- **S'entraîner** avec le langage Rust en contexte réseau
- **Explorer** les défis de l'implémentation d'un protocole de bas niveau
- **Apprendre** par la pratique plutôt que par la théorie seule

## 📚 Inspiration et Origine

Ce projet s'inspire fortement du cours de [Boot.dev](https://boot.dev) créé par **ThePrimeGen**, qui enseigne l'implémentation d'un serveur HTTP en Go. J'ai décidé de relever le défi en Rust pour :

- **Adapter** les concepts à un écosystème différent
- **Profiter** des avantages du système de types de Rust
- **Explorer** les différences d'approche entre Go et Rust
- **Personnaliser** l'implémentation selon mes besoins d'apprentissage

## 🏗️ Architecture du Projet

```
http/
├── src/
│   ├── headers/          # Parser et gestion des en-têtes HTTP
│   ├── request/          # Traitement des requêtes HTTP
│   ├── response/         # Construction des réponses HTTP
│   ├── server/           # Serveur TCP principal
│   └── main.rs          # Point d'entrée de l'application
├── Cargo.toml           # Dépendances et configuration
└── README.md           # Ce fichier
```

## ✨ Fonctionnalités Implémentées

### ✅ Fonctionnalités de Base
- **Parser de requêtes HTTP 1.1** avec validation stricte
- **Gestion des en-têtes** avec support des valeurs multiples
- **Lecture du corps de requête** basée sur Content-Length
- **Construction de réponses** avec codes d'état complets
- **Serveur TCP asynchrone** utilisant Tokio

### 🚀 Fonctionnalités Avancées
- **Compression GZIP** automatique pour les types de contenu appropriés
- **Détection automatique du type MIME** pour les réponses
- **Support JSON** natif avec détection automatique
- **Gestion d'erreurs** robuste avec types d'erreur spécifiques
- **Tests unitaires** complets pour chaque composant

## 🛠️ Technologies Utilisées

- **Rust 2024 Edition** - Langage de programmation
- **Tokio** - Runtime asynchrone
- **Flate2** - Compression GZIP
- **Chrono** - Gestion des dates
- **Serde** - (Dé)sérialisation JSON
- **Tracing** - Logging structuré

## 🚀 Utilisation

```bash
# Cloner le projet
git clone <repository-url>
cd http

# Lancer le serveur
cargo run

# Le serveur écoute sur http://127.0.0.1:3333
```

### Exemple de Test

```bash
# Tester avec curl
curl -v http://127.0.0.1:3333/
curl -H "Accept-Encoding: gzip" http://127.0.0.1:3333/
```

## 📖 Ce Que J'ai Appris

### Défis Techniques Rencontrés
1. **Parser HTTP** : La complexité de la validation des en-têtes et du format de requête
2. **Gestion asynchrone** : Comprendre le modèle de concurrence de Tokio
3. **Compression** : Implémentation correcte de GZIP avec gestion des en-têtes
4. **Gestion d'erreurs** : Création d'un système d'erreur type-safe en Rust

### Insights sur HTTP
- L'importance des retours à la ligne `\r\n` dans le protocole
- La complexité de la gestion des en-têtes multiples
- Les subtilités de la négociation de contenu et de l'encodage
- La rigueur nécessaire dans la validation des requêtes

## 🔮 Perspectives Futures

Bien que ce projet soit purement éducatif, il pourrait évoluer vers :

- **Support HTTPS** avec TLS
- **Gestion de fichiers statiques**
- **API REST complète**
- **Support WebSocket**
- **Reverse proxy** basique
- **Load balancing** simple

## 🙏 Remerciements

Un immense merci à :

- **[Boot.dev](https://boot.dev)** pour leur plateforme d'apprentissage exceptionnelle
- **[ThePrimeGen](https://github.com/ThePrimeagen)** pour son cours inspirant sur l'implémentation HTTP en Go
- **La communauté Rust** pour ses excellents outils et documentation

## 📝 Licence

Ce projet est open source et disponible sous licence MIT. Il est destiné à des fins éducatives et d'apprentissage.

---

*"La meilleure façon d'apprendre est de construire."* - Ce projet en est la preuve vivante.

**Développé avec passion pour comprendre les fondements du web moderne.**
