import { useState, useEffect, useCallback, useRef } from 'react';

interface Position {
  x: number;
  y: number;
}

interface SmartPositioningOptions {
  bubbleWidth: number;
  bubbleHeight: number;
  margin: number; // Distance minimale du curseur
  preferredSide: 'left' | 'right'; // Côté préféré
}

export function useSmartPositioning(options: SmartPositioningOptions) {
  const [position, setPosition] = useState<Position>({ x: 0, y: 0 });
  const [isVisible, setIsVisible] = useState(false);
  const cursorRef = useRef<Position>({ x: 0, y: 0 });
  const lastCursorUpdate = useRef<number>(0);
  const isPositioned = useRef<boolean>(false); // Nouveau: pour éviter le repositionnement constant

  // Détecter la position du curseur
  const updateCursorPosition = useCallback((e: MouseEvent) => {
    cursorRef.current = { x: e.clientX, y: e.clientY };
    lastCursorUpdate.current = Date.now();
  }, []);

  // Calculer la position optimale de la bulle (UNE SEULE FOIS)
  const calculateOptimalPosition = useCallback((cursor: Position): Position => {
    const { bubbleWidth, bubbleHeight, margin, preferredSide } = options;
    const screenWidth = window.innerWidth;
    const screenHeight = window.innerHeight;

    let x: number;
    let y: number;

    // Position Y : éviter de couper le curseur
    if (cursor.y + margin + bubbleHeight > screenHeight) {
      // Placer au-dessus du curseur
      y = Math.max(0, cursor.y - bubbleHeight - margin);
    } else {
      // Placer en dessous du curseur
      y = cursor.y + margin;
    }

    // Position X : côté préféré ou côté disponible
    if (preferredSide === 'right') {
      if (cursor.x + margin + bubbleWidth <= screenWidth) {
        // Assez de place à droite
        x = cursor.x + margin;
      } else if (cursor.x - margin - bubbleWidth >= 0) {
        // Pas assez de place à droite, aller à gauche
        x = cursor.x - margin - bubbleWidth;
      } else {
        // Pas assez de place des deux côtés, centrer
        x = Math.max(0, (screenWidth - bubbleWidth) / 2);
      }
    } else {
      if (cursor.x - margin - bubbleWidth >= 0) {
        // Assez de place à gauche
        x = cursor.x - margin - bubbleWidth;
      } else if (cursor.x + margin + bubbleWidth <= screenWidth) {
        // Pas assez de place à gauche, aller à droite
        x = cursor.x + margin;
      } else {
        // Pas assez de place des deux côtés, centrer
        x = Math.max(0, (screenWidth - bubbleWidth) / 2);
      }
    }

    return { x, y };
  }, [options]);

  // Écouter les mouvements de souris (pour la position initiale seulement)
  useEffect(() => {
    document.addEventListener('mousemove', updateCursorPosition);
    return () => document.removeEventListener('mousemove', updateCursorPosition);
  }, [updateCursorPosition]);

  // Fonctions publiques
  const showBubble = useCallback((initialPosition?: Position) => {
    if (!isPositioned.current) {
      const pos = initialPosition || calculateOptimalPosition(cursorRef.current);
      setPosition(pos);
      isPositioned.current = true;
    }
    setIsVisible(true);
  }, [calculateOptimalPosition]);

  const hideBubble = useCallback(() => {
    setIsVisible(false);
    isPositioned.current = false; // Reset pour le prochain affichage
  }, []);

  const forceReposition = useCallback(() => {
    if (isVisible) {
      const newPos = calculateOptimalPosition(cursorRef.current);
      setPosition(newPos);
    }
  }, [isVisible, calculateOptimalPosition]);

  return {
    position,
    isVisible,
    showBubble,
    hideBubble,
    forceReposition,
    isInterfering: false, // Supprimé car plus de suivi constant
  };
}
