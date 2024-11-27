'use client';

import * as React from "react";
import { X, Wand2 } from "lucide-react";
import Image from "next/image";
import { Button } from "./button";

interface FeelingLuckyModalProps {
  onClose: () => void;
}

export function FeelingLuckyModal({ onClose }: FeelingLuckyModalProps) {
  return (
    <div className="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
         onClick={(e) => {
           if (e.target === e.currentTarget) onClose();
         }}>
      <div className="fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg"
           onClick={(e) => e.stopPropagation()}>
        <div className="flex flex-col space-y-1.5 text-center relative">
          <button 
            onClick={onClose}
            className="absolute right-0 top-0 p-2 hover:bg-muted rounded-sm transition-colors"
          >
            <X className="h-4 w-4" />
          </button>
          
          <div className="mx-auto relative w-24 h-24 mb-4">
            <div className="absolute inset-0 animate-pulse">
              <Image
                src="/images/mascot.jpg"
                alt="Sonoma Search Mascot"
                className="object-contain"
                width={96}
                height={96}
                priority
              />
            </div>
            <Wand2 className="absolute -right-2 -top-2 h-6 w-6 text-primary animate-bounce" />
          </div>

          <h2 className="text-lg font-semibold">Coming Soon!</h2>
          <p className="text-sm text-muted-foreground max-w-md mx-auto">
            We're working on something magical! Soon you'll be able to discover interesting content with just one click.
          </p>

          <div className="pt-4 pb-2">
            <div className="w-full h-px bg-gradient-to-r from-transparent via-muted-foreground/25 to-transparent" />
          </div>

          <p className="text-xs text-muted-foreground italic">
            "I'm Feeling Lucky" will take you directly to the most relevant page for your search.
          </p>

          <div className="flex justify-center mt-6">
            <Button variant="outline" onClick={onClose}>Got it!</Button>
          </div>
        </div>
      </div>
    </div>
  );
}
