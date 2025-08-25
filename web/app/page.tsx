'use client'

import { ChatInterface } from '@/components/ChatInterface'
import { WelcomeScreen } from '@/components/WelcomeScreen'
import { useState } from 'react'

export default function Home() {
  const [hasStartedChat, setHasStartedChat] = useState(false)

  return (
    <main className="flex flex-col h-screen bg-background">
      {!hasStartedChat ? (
        <WelcomeScreen onStartChat={() => setHasStartedChat(true)} />
      ) : (
        <ChatInterface />
      )}
    </main>
  )
}
