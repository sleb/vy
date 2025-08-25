'use client'

import { useState } from 'react'

interface WelcomeScreenProps {
  onStartChat: () => void
}

export function WelcomeScreen({ onStartChat }: WelcomeScreenProps) {
  const [isLoading, setIsLoading] = useState(false)

  const handleStartChat = () => {
    setIsLoading(true)
    // Add a small delay for UX
    setTimeout(() => {
      onStartChat()
    }, 300)
  }

  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-6 text-center">
      <div className="max-w-md mx-auto space-y-8">
        {/* Logo/Icon */}
        <div className="w-24 h-24 mx-auto bg-primary rounded-full flex items-center justify-center">
          <span className="text-3xl font-bold text-primary-foreground">Vy</span>
        </div>

        {/* Welcome Text */}
        <div className="space-y-4">
          <h1 className="text-4xl font-bold text-foreground">
            Welcome to Vy
          </h1>
          <p className="text-lg text-muted-foreground leading-relaxed">
            Your personal AI assistant with memory, real-time search, and natural conversation.
          </p>
        </div>

        {/* Features List */}
        <div className="space-y-3 text-left">
          <div className="flex items-center space-x-3">
            <div className="w-2 h-2 bg-primary rounded-full"></div>
            <span className="text-muted-foreground">Remembers your conversations</span>
          </div>
          <div className="flex items-center space-x-3">
            <div className="w-2 h-2 bg-primary rounded-full"></div>
            <span className="text-muted-foreground">Real-time Google search</span>
          </div>
          <div className="flex items-center space-x-3">
            <div className="w-2 h-2 bg-primary rounded-full"></div>
            <span className="text-muted-foreground">Personalized responses</span>
          </div>
        </div>

        {/* Start Button */}
        <button
          onClick={handleStartChat}
          disabled={isLoading}
          className="w-full bg-primary text-primary-foreground px-8 py-4 rounded-lg font-semibold text-lg hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading ? (
            <div className="flex items-center justify-center space-x-2">
              <div className="w-4 h-4 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin"></div>
              <span>Starting...</span>
            </div>
          ) : (
            "Start Chatting"
          )}
        </button>

        {/* Version Info */}
        <p className="text-xs text-muted-foreground">
          Powered by Rust & Next.js
        </p>
      </div>
    </div>
  )
}
