import Link from 'next/link';

export default function Home() {
  return (
    <div className="min-h-screen">
      {/* Hero */}
      <section className="relative flex flex-col items-center justify-center min-h-screen px-4 text-center">
        <div className="absolute inset-0 bg-gradient-to-b from-forkit-green/5 to-transparent" />
        <div className="relative z-10 max-w-4xl">
          <h1 className="text-6xl md:text-8xl font-bold mb-6">
            <span className="text-forkit-green">Fork</span>It
          </h1>
          <p className="text-xl md:text-2xl text-gray-300 mb-4">
            Decentralized Food Delivery on Solana
          </p>
          <p className="text-lg text-gray-400 mb-12 max-w-2xl mx-auto">
            No middlemen. No 30% fees. Just restaurants, drivers, and customers
            connected through trustless escrow smart contracts.
            <strong className="text-forkit-green"> 0.02% protocol fee.</strong>
          </p>

          {/* Role CTAs */}
          <div className="grid md:grid-cols-3 gap-6 max-w-3xl mx-auto">
            <Link
              href="/restaurant/dashboard"
              className="group p-8 rounded-2xl bg-forkit-slate border border-gray-700 hover:border-forkit-green transition-all"
            >
              <div className="text-4xl mb-4">🍕</div>
              <h3 className="text-xl font-bold mb-2 group-hover:text-forkit-green transition-colors">
                I&apos;m a Restaurant
              </h3>
              <p className="text-gray-400 text-sm">
                List your menu, accept orders, keep 99.98% of your revenue.
              </p>
            </Link>

            <Link
              href="/driver/dashboard"
              className="group p-8 rounded-2xl bg-forkit-slate border border-gray-700 hover:border-forkit-green transition-all"
            >
              <div className="text-4xl mb-4">🚗</div>
              <h3 className="text-xl font-bold mb-2 group-hover:text-forkit-green transition-colors">
                I&apos;m a Driver
              </h3>
              <p className="text-gray-400 text-sm">
                Set your own zones and pricing. Earn what you deserve.
              </p>
            </Link>

            <Link
              href="/customer/browse"
              className="group p-8 rounded-2xl bg-forkit-slate border border-gray-700 hover:border-forkit-green transition-all"
            >
              <div className="text-4xl mb-4">🍽️</div>
              <h3 className="text-xl font-bold mb-2 group-hover:text-forkit-green transition-colors">
                I&apos;m Hungry
              </h3>
              <p className="text-gray-400 text-sm">
                Browse local restaurants. Pay with stablecoins. No platform markup.
              </p>
            </Link>
          </div>
        </div>
      </section>

      {/* How it Works */}
      <section className="py-24 px-4 bg-forkit-slate/50">
        <div className="max-w-5xl mx-auto">
          <h2 className="text-4xl font-bold text-center mb-16">
            How <span className="text-forkit-green">ForkIt</span> Works
          </h2>

          <div className="grid md:grid-cols-3 gap-8">
            <div className="text-center p-6">
              <div className="w-16 h-16 rounded-full bg-forkit-green/20 text-forkit-green text-2xl font-bold flex items-center justify-center mx-auto mb-4">
                1
              </div>
              <h3 className="text-lg font-semibold mb-2">Order &amp; Escrow</h3>
              <p className="text-gray-400 text-sm">
                Customer places order. Payment + security deposit locked in a Solana escrow smart contract.
              </p>
            </div>

            <div className="text-center p-6">
              <div className="w-16 h-16 rounded-full bg-forkit-amber/20 text-forkit-amber text-2xl font-bold flex items-center justify-center mx-auto mb-4">
                2
              </div>
              <h3 className="text-lg font-semibold mb-2">Code Confirmation Chain</h3>
              <p className="text-gray-400 text-sm">
                Restaurant → CODE_A → Driver → CODE_B → Customer. Cryptographic handoff at every step.
              </p>
            </div>

            <div className="text-center p-6">
              <div className="w-16 h-16 rounded-full bg-forkit-green/20 text-forkit-green text-2xl font-bold flex items-center justify-center mx-auto mb-4">
                3
              </div>
              <h3 className="text-lg font-semibold mb-2">Instant Settlement</h3>
              <p className="text-gray-400 text-sm">
                Once confirmed, funds release instantly. Restaurant and driver paid. Deposit returned. 0.02% fee to protocol.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Comparison */}
      <section className="py-24 px-4">
        <div className="max-w-3xl mx-auto text-center">
          <h2 className="text-4xl font-bold mb-12">The Numbers Don&apos;t Lie</h2>
          <div className="grid grid-cols-2 gap-8">
            <div className="p-8 rounded-2xl bg-forkit-red/10 border border-forkit-red/30">
              <p className="text-5xl font-bold text-forkit-red mb-2">15-30%</p>
              <p className="text-gray-400">Uber Eats / DoorDash take per order</p>
            </div>
            <div className="p-8 rounded-2xl bg-forkit-green/10 border border-forkit-green/30">
              <p className="text-5xl font-bold text-forkit-green mb-2">0.02%</p>
              <p className="text-gray-400">ForkIt protocol fee</p>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-12 px-4 border-t border-gray-800">
        <div className="max-w-5xl mx-auto flex flex-col md:flex-row items-center justify-between gap-4">
          <div className="flex items-center gap-2">
            <span className="text-xl">🍴</span>
            <span className="font-bold">ForkIt</span>
            <span className="text-gray-500 text-sm">— Decentralized Food Delivery Protocol</span>
          </div>
          <div className="text-gray-500 text-sm">
            Built on Solana • Powered by community
          </div>
        </div>
      </footer>
    </div>
  );
}
