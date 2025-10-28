# Bugs Critiques - ShadowLearn

**Date:** 2025-10-28  
**Status:** 7/7 bugs identifiés

---

## 🔴 BUG #1: TriggerBubble ne s'affiche pas
**Feature:** TriggerBubble  
**Symptôme:** Petite bulle circulaire jamais visible  
**Impact:** Feature inutilisable  
**Priorité:** HAUTE  
**Status:** ❌ NON FIXÉ

---

## 🔴 BUG #2: OpportunityToast ne s'affiche pas (ou mal)
**Feature:** OpportunityToast  
**Symptôme:** Toast "J'ai une idée" ne s'affiche pas ou se positionne mal  
**Impact:** Feature inutilisable  
**Priorité:** HAUTE  
**Status:** ⚠️ PARTIELLEMENT FIXÉ (affichage OK, positionnement à valider)

---

## ✅ BUG #3: SlashCommands n'exécutent pas les commandes
**Feature:** SlashCommands  
**Symptôme:** Palette s'affiche, navigation OK, mais envoi ne déclenche rien  
**Impact:** Feature inutilisable  
**Priorité:** HAUTE  
**Status:** ✅ FIXÉ
**Solution:** Ajout du callback `onCommandResult` + appel backend + affichage résultat dans chat

---

## 🔴 BUG #4: QuickActions ne réagissent pas au clic
**Feature:** QuickActions  
**Symptôme:** Boutons visibles mais aucune réaction au clic  
**Impact:** Feature inutilisable  
**Priorité:** HAUTE  
**Status:** ❌ NON FIXÉ

---

## 🔴 BUG #5: SmartPills ne s'affichent jamais
**Feature:** SmartPills  
**Symptôme:** Capsules jamais visibles (spontané, inactivité, événement manuel)  
**Impact:** Feature inutilisable  
**Priorité:** HAUTE  
**Status:** ❌ NON FIXÉ

---

## 🔴 BUG #6: StreakTracker invisible
**Feature:** StreakTracker  
**Symptôme:** Badge 🔥 et compteur jamais visibles  
**Impact:** Feature inutilisable  
**Priorité:** MOYENNE  
**Status:** ❌ NON FIXÉ

---

## 🔴 BUG #7: PersonalitySelector ne reflète pas le mode actif
**Feature:** PersonalitySelector  
**Symptôme:** Badge visible, sélection ne se ferme pas, UI ne reflète pas le mode  
**Impact:** Feature partiellement fonctionnelle  
**Priorité:** MOYENNE  
**Status:** ❌ NON FIXÉ

---

## 📊 Résumé
- **Total:** 7 bugs critiques
- **Fixés:** 1 (SlashCommands)
- **En cours:** 1 (OpportunityToast)
- **Non fixés:** 5

## 🎯 Plan d'action
1. ⚠️ OpportunityToast (affichage OK, positionnement en cours)
2. ✅ SlashCommands (FIXÉ - backend + affichage résultat)
3. ⏳ NEXT → QuickActions (handlers manquants)
4. Fixer SmartPills (emission backend manquante)
5. Fixer TriggerBubble (problème d'affichage)
6. Fixer StreakTracker (positionnement)
7. Fixer PersonalitySelector (UI state)

