'use client';

import * as React from "react";
import { X, Search, Bot, Cpu, Sparkles } from "lucide-react";
import Image from "next/image";
import { Button } from "./button";

interface InfoModalProps {
  onClose: () => void;
}

export function InfoModal({ onClose }: InfoModalProps) {
  return (
    <div className="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
         onClick={(e) => {
           if (e.target === e.currentTarget) onClose();
         }}>
      <div className="fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg"
           onClick={(e) => e.stopPropagation()}>
        <div className="flex flex-col space-y-1.5 relative">
          <button 
            onClick={onClose}
            className="absolute right-0 top-0 p-2 hover:bg-muted rounded-sm transition-colors"
          >
            <X className="h-4 w-4" />
          </button>
          
          <div className="flex items-center gap-2 mb-4">
            <div className="relative w-12 h-12">
              <Image
                src="/images/mascot.jpg"
                alt="Sonoma Search Mascot"
                className="object-contain"
                width={48}
                height={48}
                priority
              />
            </div>
            <div>
              <h2 className="text-lg font-semibold">About Sonoma Search</h2>
              <p className="text-sm text-muted-foreground">Your intelligent web companion</p>
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex items-start gap-3 p-3 rounded-lg border">
              <Search className="h-5 w-5 mt-0.5 text-primary" />
              <div>
                <h3 className="font-medium">Smart Search</h3>
                <p className="text-sm text-muted-foreground">
                  Advanced search algorithm that understands context and relevance
                </p>
              </div>
            </div>

            <div className="flex items-start gap-3 p-3 rounded-lg border">
              <Bot className="h-5 w-5 mt-0.5 text-primary" />
              <div>
                <h3 className="font-medium">Adaptive Crawler</h3>
                <p className="text-sm text-muted-foreground">
                  Intelligent web crawler that finds the most relevant content for your queries
                </p>
              </div>
            </div>

            <div className="flex items-start gap-3 p-3 rounded-lg border">
              <Cpu className="h-5 w-5 mt-0.5 text-primary" />
              <div>
                <h3 className="font-medium">Real-time Processing</h3>
                <p className="text-sm text-muted-foreground">
                  Results are processed and ranked in real-time for maximum freshness
                </p>
              </div>
            </div>

            <div className="flex items-start gap-3 p-3 rounded-lg border">
              <Sparkles className="h-5 w-5 mt-0.5 text-primary" />
              <div>
                <h3 className="font-medium">Quality Scoring</h3>
                <p className="text-sm text-muted-foreground">
                  Each page is scored based on content quality and relevance
                </p>
              </div>
            </div>
          </div>

          <div className="flex justify-end mt-6">
            <Button variant="outline" onClick={onClose}>Close</Button>
          </div>
        </div>
      </div>
    </div>
  );
}
