import type { Metadata } from "next"
import { Inter } from "next/font/google"
import "./globals.css"
import { ThemeProvider } from "../components/theme/ThemeProvider"

const inter = Inter({ 
  subsets: ["latin"],
  variable: '--font-inter',
})

export const metadata: Metadata = {
  title: "Sonoma Search",
  description: "A modern search engine with adaptive crawling",
  icons: {
    icon: [
      {
        url: "/images/mascot.jpg",
        sizes: "32x32",
        type: "image/jpeg"
      },
      {
        url: "/images/mascot.jpg",
        sizes: "16x16",
        type: "image/jpeg"
      }
    ],
    apple: {
      url: "/images/mascot.jpg",
      sizes: "180x180",
      type: "image/jpeg"
    },
    shortcut: "/images/mascot.jpg"
  }
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={inter.className} suppressHydrationWarning>
        <ThemeProvider>
          {children}
        </ThemeProvider>
      </body>
    </html>
  )
}